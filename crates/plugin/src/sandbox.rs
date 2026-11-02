// Modified by FastTrackStudios: Cargo features to leave out wasmi, krilla, craft-font-store and vectordrawable (see FORK.md).
//! The sandbox: a plugin module runs in a metered WebAssembly interpreter (wasmi; no JIT) with a
//! memory cap, and sees the outside world only through the host functions below. There is no WASI:
//! no file system, network, clock or randomness.
//!
//! # ABI (version 4; version 1 is the first seven imports, version 2 adds panels and settings,
//! version 3 widgets, version 4 `save_file`)
//!
//! The module exports its `memory` and `figcraft_run(input_len: i32) -> i32`. Host functions are
//! imported from the module `figcraft`; strings are UTF-8 `(ptr, len)` pairs in the plugin's memory.
//!
//! | Import | Signature | Does |
//! |---|---|---|
//! | `call` | `(cmd_ptr, cmd_len, params_ptr, params_len) -> i32` | Runs a command with JSON params. 0: ok, the reply holds the result JSON; 1: refused or failed, the reply holds the message. |
//! | `selection` | `() -> i32` | The reply becomes the selected layer ids (a JSON array); returns its length. |
//! | `reply_len` | `() -> i32` | Length of the reply. |
//! | `reply_read` | `(ptr, len) -> i32` | Copies up to `len` bytes of the reply to `ptr`; returns the count. |
//! | `log` | `(ptr, len)` | A line for the run's log. |
//! | `notify` | `(ptr, len)` | A short message shown to the user. |
//! | `set_output` | `(ptr, len)` | The run's result (JSON or text), or its error message when it fails. |
//! | `ui_show` | `(ptr, len) -> i32` | Shows (or replaces) the plugin's panel: a [`Panel`] as JSON. 1: invalid, the reply says why. |
//! | `ui_close` | `()` | Closes the panel. |
//! | `setting_get` | `(key_ptr, key_len) -> i32` | 0: the reply holds the stored JSON; 1: no such setting. |
//! | `setting_set` | `(key_ptr, key_len, value_ptr, value_len) -> i32` | Stores JSON under the key. 1: refused, the reply says why. |
//! | `setting_delete` | `(key_ptr, key_len)` | Forgets a setting. |
//! | `widget_render` | `(ptr, len) -> i32` | Draws the widget being called back: a [`WidgetRender`] as JSON. 1: invalid, the reply says why. |
//! | `save_file` | `(name_ptr, name_len, mime_ptr, mime_len, data_ptr, data_len) -> i32` | Offers a file to save (needs `permissions.saveFile`). The host asks the user where it goes; the plugin never gets a path. 0: accepted, 1: refused, the reply says why. |
//!
//! When `figcraft_run` starts, the reply holds the input JSON (`input_len` bytes):
//! `{"api", "command", "params", "selection"}`, plus `event` and `values` when a panel calls the
//! plugin back, or `widget` (`{id, state, event}`) when a widget does. It returns 0 on success; anything else fails the run.
//!
//! An [`Instance`] keeps the module's memory between calls, so a plugin with a panel open keeps its
//! state from one callback to the next. Every call gets the full [`Limits`] again.

use crate::manifest::Panel;
use crate::widget::WidgetRender;

/// The module host functions are imported from.
pub const IMPORT_MODULE: &str = "figcraft";
/// The exported entry point.
pub const ENTRY: &str = "figcraft_run";


/// What a run may use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Instruction budget (wasmi fuel). Each host call costs [`Limits::call_fuel`] more,
    /// plus one fuel per byte copied out of or into guest memory.
    pub fuel: u64,
    pub call_fuel: u64,
    /// Linear memory cap in bytes.
    pub memory_bytes: usize,
    /// Largest module accepted.
    pub module_bytes: usize,
    /// Host calls per run.
    pub max_calls: u32,
    /// Largest string passed either way (command params, results, logs).
    pub max_message: usize,
    /// Log lines kept and forwarded per run. Exceeding any log limit fails the run.
    pub max_logs: usize,
    /// Largest log line in UTF-8 bytes, also subject to [`Limits::max_message`].
    pub max_log_line_bytes: usize,
    /// Total log bytes kept and forwarded per run (excluding string/vector overhead).
    pub max_log_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            fuel: 1_000_000_000,
            call_fuel: 10_000,
            memory_bytes: 64 * 1024 * 1024,
            module_bytes: 16 * 1024 * 1024,
            max_calls: 100_000,
            max_message: 16 * 1024 * 1024,
            max_logs: 1000,
            max_log_line_bytes: 8 * 1024,
            max_log_bytes: 1024 * 1024,
        }
    }
}

/// Why a run failed.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PluginError {
    #[error("not a valid plugin module: {0}")]
    InvalidModule(String),
    #[error("the plugin ran out of its instruction budget (an endless loop?)")]
    OutOfFuel,
    #[error("the plugin used more than its memory limit")]
    OutOfMemory,
    #[error("the plugin crashed: {0}")]
    Trap(String),
    #[error("the plugin went over a limit: {0}")]
    Limit(String),
    #[error("{0}")]
    Failed(String),
}

/// What the sandbox asks of its host (the engine): run commands, read the selection, log, keep
/// settings.
pub trait Host {
    /// Run `command` with `params` (JSON text): the result JSON, or why it was refused or failed.
    fn call(&mut self, command: &str, params: &str) -> Result<String, String>;
    /// The selected layer ids as a JSON array.
    fn selection(&mut self) -> String;
    /// A message for the user.
    fn notify(&mut self, message: &str);
    /// An admitted line of the run's log (also in [`RunOutput::logs`]). Log limits apply before
    /// this is called, including on runs that later fail.
    fn log(&mut self, _line: &str) {}
    /// The plugin's setting `key` (JSON text).
    fn setting(&mut self, _key: &str) -> Option<String> {
        None
    }
    /// Store (`Some`) or forget (`None`) setting `key`, or say why not.
    fn set_setting(&mut self, _key: &str, _value: Option<&str>) -> Result<(), String> {
        Err("settings aren't available here".into())
    }
    /// Offer the file `name` (MIME type `mime`) for saving, or say why not. The host picks the
    /// destination; the plugin never learns it.
    fn save_file(&mut self, _name: &str, _mime: &str, _data: &[u8]) -> Result<(), String> {
        Err("saving files isn't available here".into())
    }
}

/// What a run asked of the panel.
#[derive(Clone, Debug, PartialEq)]
pub enum UiChange {
    /// Show this panel (or replace the one showing).
    Show(Panel),
    Close,
}

/// A finished run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunOutput {
    /// What the plugin passed to `set_output` (empty when nothing).
    pub output: String,
    pub logs: Vec<String>,
    /// Host calls made.
    pub calls: u32,
    /// Fuel used.
    pub fuel: u64,
    /// The last `ui_show` / `ui_close` of the run.
    pub ui: Option<UiChange>,
    /// The last `widget_render` of the run.
    pub widget: Option<WidgetRender>,
}

#[cfg(feature = "sandbox")]
mod wasm;
#[cfg(feature = "sandbox")]
pub use wasm::{Instance, check_module, run};
#[cfg(not(feature = "sandbox"))]
mod off;
#[cfg(not(feature = "sandbox"))]
pub use off::{Instance, check_module, run};
