# FastTrackStudios fork of figcraft

This is FastTrackStudios/figcraft, a fork of storytold/figcraft. The `fts` branch carries a few
patches on top of an upstream **release tag**. FastTrackStudios/frame depends on it by tag
(`fts-v<upstream>-<n>`), only for the editor crates.

Licence: figcraft is `MIT OR Apache-2.0`; we use it under Apache-2.0 and keep `LICENSE-APACHE`. A
file we changed carries a "Modified by FastTrackStudios" line (Apache-2.0 §4(b)).

## Policy

- Nothing goes upstream without Cody's yes. Never push to storytold/figcraft. Upstream
  contributions are PRs from this fork, once agreed.
- Keep `fts` as small as it can be. Each patch is written so it could go upstream.
- Every patch commit is prefixed `fts:` and listed below, with its upstream PR if there is one.

## Patches

| # | Commit subject | What | Upstream PR |
|---|----------------|------|-------------|
| 1 | `fts: engine features to drop pdf, plugins, font-download, vectordrawable` | `figcraft-engine` gets Cargo features `pdf` (krilla), `plugins` (wasmi), `font-download` (craft-font-store) and `vectordrawable`. All four are on by default, so upstream's behaviour is unchanged. `figcraft-plugin` gets a `sandbox` feature (default on) around its wasmi code; without it the manifest, bundle and widget types stay and no module runs. The workspace dependency on `figcraft-plugin` turns default features off, so the server enables `sandbox` itself. Export to PDF/VectorDrawable and plugin runs return an "isn't built into this program" error when their feature is off. | none yet |

A host that only edits documents depends on `figcraft-engine` with `default-features = false`.
`craft-font-catalog` still comes in through `figcraft-text` (it only lists font folders).

## Rebasing onto a new upstream release

At most every two weeks, or when we need a fix. Rebase onto release tags, not `main`: upstream
ships about 120 commits a day.

1. `git fetch upstream --tags`
2. `git rebase --onto vX.Y.Z <old-base-tag> fts`
3. `cargo xtask layers`, then upstream's tests (`cargo test -p figcraft-engine -p figcraft-plugin`),
   plus `cargo check -p figcraft-engine --no-default-features` to prove the slim build still holds
   (`cargo tree` must show no wasmi, krilla or craft-font-store).
4. Run Frame's checks (`ids.styx` and the parity suite) against the result.
5. Tag it `fts-vX.Y.Z-1` (`-n` counts our re-tags of the same upstream release), push `fts` and the tag.
6. Bump the tag in Frame in its own PR.

## Tags

- `fts-v0.14.0-1`: upstream `v0.14.0-4-gbd920fc7` plus patch 1.
