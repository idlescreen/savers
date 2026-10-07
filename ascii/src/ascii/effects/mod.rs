// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! The six built-in text effects.
//!
//! Each is a small state machine over [`CellState`]. All of them mutate
//! `current` in place and allocate nothing — see `AGENTS.md`'s steady-state
//! zero-allocation rule.

pub mod decrypt;
pub mod led;
pub mod matrix;
pub mod scramble;
pub mod shower;
pub mod wave;
