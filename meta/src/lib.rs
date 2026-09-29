// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! # IdleScreen Plugins All Meta-Crate
//!
//! This is a packaging-only placeholder library for the `idle-plugins-all`
//! Debian package. It does not export any functions or contain active Rust logic.
//!
//! ## Discovery
//!
//! The `idle-runner` automatically discovers screensaver binary libraries located in
//! `/usr/libexec/idle/screensavers/` (installed by the individual screen packages) and
//! presents them dynamically to the user session applet.

pub mod plugins;
pub use plugins::*;
