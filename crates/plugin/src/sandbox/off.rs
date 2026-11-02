//! The sandbox when the `sandbox` feature is off: the types exist, but no plugin can run.
// Modified by FastTrackStudios: added with the `sandbox` feature so hosts can leave out wasmi.

use super::{Host, Limits, PluginError, RunOutput};

const OFF: &str = "plugins are not built into this program";

/// Always fails: there is no sandbox to check a module with.
pub fn check_module(_bytes: &[u8], _limits: &Limits) -> Result<(), PluginError> {
    Err(PluginError::InvalidModule(OFF.into()))
}

/// Always fails: there is no sandbox to run a module in.
pub fn run(_bytes: &[u8], _input: &str, _limits: &Limits, _host: &mut dyn Host) -> Result<RunOutput, PluginError> {
    Err(PluginError::InvalidModule(OFF.into()))
}

/// A plugin module that can never start.
#[derive(Debug)]
pub struct Instance(());

impl Instance {
    /// Always fails.
    pub fn new(_bytes: &[u8], _limits: &Limits) -> Result<Self, PluginError> {
        Err(PluginError::InvalidModule(OFF.into()))
    }

    /// Always fails.
    pub fn call(&mut self, _input: &str, _host: &mut dyn Host) -> Result<RunOutput, PluginError> {
        Err(PluginError::InvalidModule(OFF.into()))
    }
}
