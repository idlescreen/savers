// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Shower — each cell falls on its own speed, cycling noise → blank → art, so
//! the block appears to be rained on rather than wiped.

use super::super::cell_state::CellState;
use crate::runner::LcgRng;

/// Seconds for one cell to complete a full cycle.
const CYCLE: f32 = 4.0;
/// Per-cell speed multiplier range: 1.4 (slowest) ..=4.6 (fastest).
const SPEED_MIN: f32 = 1.4;
const SPEED_SPAN: f32 = 3.2;
/// Cycle fractions: noise, then blank, then art.
const NOISE_END: f32 = 0.35;
const BLANK_END: f32 = 0.55;

pub fn advance(st: &mut CellState, _dt: f32, t: f32, ramp: &[char], rng: &mut LcgRng) {
    if ramp.is_empty() {
        return;
    }
    for i in 0..st.current.len() {
        let target = st.target[i];
        if target == ' ' {
            st.current[i] = ' ';
            continue;
        }
        let phase = st.phase[i];
        let local = ((t * (SPEED_MIN + phase * SPEED_SPAN) + phase * CYCLE) % CYCLE) / CYCLE;
        st.current[i] = if local < NOISE_END {
            ramp[rng.next_usize(ramp.len())]
        } else if local < BLANK_END {
            ' '
        } else {
            target
        };
    }
}
