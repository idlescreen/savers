// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Decrypt — random ramp characters resolve in place, cell by cell, until the
//! art is fully legible. Each cell resolves at its own rate via its `phase`.

use super::super::cell_state::CellState;
use crate::runner::LcgRng;

/// Seconds a cell takes to resolve at unit stagger.
const RESOLVE_SECONDS: f32 = 6.0;

/// Per-cell rate multiplier range: 1.0 (slowest) ..=2.4 (fastest).
const STAGGER_MAX: f32 = 1.4;

pub fn advance(st: &mut CellState, dt: f32, _t: f32, ramp: &[char], rng: &mut LcgRng) {
    if ramp.is_empty() || dt <= 0.0 {
        return;
    }
    let rate = 1.0 / RESOLVE_SECONDS;

    for i in 0..st.current.len() {
        let target = st.target[i];
        if target == ' ' {
            st.current[i] = ' ';
            st.settled[i] = 1.0;
            continue;
        }
        if st.settled[i] < 1.0 {
            let boost = 1.0 + st.phase[i] * STAGGER_MAX;
            st.settled[i] = (st.settled[i] + dt * rate * boost).min(1.0);
        }
        st.current[i] = if st.settled[i] >= 1.0 {
            target
        } else {
            ramp[rng.next_usize(ramp.len())]
        };
    }
}