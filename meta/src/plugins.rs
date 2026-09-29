// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Catalog of screensavers packaged by the metapackage.

/// List of screensavers bundled in `idle-plugins-all`.
pub const PLUGINS: &[&str] = &[
    "beams", "bursts", "chaos", "cosmos", "glyphs", "gnats", "hearth", "radar", "ripple", "storm",
];

/// Returns the count of default screensaver plugins.
pub fn plugin_count() -> usize {
    PLUGINS.len()
}
