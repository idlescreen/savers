// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! # IdleScreen Plugins All Meta-Crate
//!
//! This is a packaging-only placeholder library for the `idle-plugins-all`
//! Debian package. It defines the catalog of default screensavers packaged
//! by the metapackage.
//!
//! ## Discovery
//!
//! The `idle-runner` automatically discovers screensaver binary libraries located in
//! `/usr/libexec/idle/screensavers/` (installed by the individual screen packages) and
//! presents them dynamically to the user session applet.

/// List of screensavers bundled in `idle-plugins-all`.
pub const PLUGINS: &[&str] = &[
    "beams", "bursts", "chaos", "cosmos", "glyphs", "gnats", "hearth", "radar", "ripple", "storm",
];

/// Returns the count of default screensaver plugins.
pub fn plugin_count() -> usize {
    PLUGINS.len()
}

pub mod plugins {
    pub use super::{PLUGINS, plugin_count};
}
