// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Led — an ignition front expands outward from the centre of the grid,
//! revealing the art behind it while unlit cells hold a dim ramp character.

use super::super::cell_state::CellState;
use crate::runner::LcgRng;

/// Sweep cycles per second.
const SWEEP_HZ: f32 = 0.11;

/// Fraction of each sweep spent lit.
const DUTY: f32 = 0.72;

pub fn advance(st: &mut CellState, dt: f32, _t: f32, ramp: &[char], rng: &mut LcgRng) {
    let (cols, rows) = (st.cols, st.rows);
    if cols == 0 || rows == 0 || ramp.is_empty() {
        return;
    }
    let cx = (cols as f32 - 1.0) / 2.0;
    let cy = (rows as f32 - 1.0) / 2.0;
    let max_radius = (cx.mul_add(cx, cy * cy)).sqrt().max(1.0);

    // Sweep wraps; `progress` is the owned scalar, so no per-effect buffer.
    st.progress = (st.progress + dt * SWEEP_HZ) % 1.0;
    let front = st.progress * (1.0 / DUTY).min(1.0 / DUTY);

    for y in 0..rows {
        let dy = y as f32 - cy;
        for x in 0..cols {
            let i = y * cols + x;
            let dx = x as f32 - cx;
            let distance = (dx.mul_add(dx, dy * dy)).sqrt() / max_radius;
            st.current[i] = if distance <= front {
                st.target[i]
            } else {
                ramp[rng.next_usize(ramp.len())]
            };
        }
    }
}
