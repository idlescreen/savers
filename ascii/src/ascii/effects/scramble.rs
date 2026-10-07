// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Scramble — the art shimmers. Every cell independently flips between a ramp
//! character and its true glyph on its own phase-offset cycle, so the block
//! is legible between bursts of noise.

use super::super::cell_state::CellState;
use crate::runner::LcgRng;

/// Seconds one cell spends in a single noise/burst cycle.
const CYCLE: f32 = 2.6;
/// Fraction of each cycle spent showing noise.
const NOISE_SHARE: f32 = 0.62;

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
        let local = ((t + st.phase[i] * CYCLE) % CYCLE) / CYCLE;
        st.current[i] = if local < NOISE_SHARE {
            ramp[rng.next_usize(ramp.len())]
        } else {
            target
        };
    }
}