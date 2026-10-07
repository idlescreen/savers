// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Wave — the art is sampled through a horizontal sine displacement, so the
//! block ripples. Cells displaced past the grid edge read as blank.

use super::super::cell_state::CellState;
use crate::runner::LcgRng;

/// Vertical displacement, in rows, at the wave crest.
const AMPLITUDE: f32 = 1.6;
/// Radians per second.
const SPEED: f32 = 1.9;
/// Horizontal phase spread per column, in radians.
const SLOPE: f32 = 0.28;

pub fn advance(st: &mut CellState, _dt: f32, t: f32, _ramp: &[char], _rng: &mut LcgRng) {
    let (cols, rows) = (st.cols, st.rows);
    if cols == 0 || rows == 0 {
        return;
    }

    for x in 0..cols {
        let offset = ((t * SPEED + x as f32 * SLOPE).sin()) * AMPLITUDE;
        for y in 0..rows {
            let i = y * cols + x;
            let source = (y as f32 + offset).round();
            st.current[i] = if source >= 0.0 && (source as usize) < rows {
                st.target[source as usize * cols + x]
            } else {
                ' '
            };
        }
    }
}
