// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Matrix — a rain head sweeps down each column, leaving revealed art behind
//! it and blank cells still ahead of it.

use super::super::cell_state::CellState;
use crate::runner::LcgRng;

/// Rows of travel per second, and the width of the rain trail.
const SPEED: f32 = 11.0;
const TRAIL: i32 = 4;
/// Blank cells kept above the head so the reveal reads as top-down.
const LEAD_IN: i32 = 7;

pub fn advance(st: &mut CellState, _dt: f32, t: f32, ramp: &[char], rng: &mut LcgRng) {
    let (cols, rows) = (st.cols, st.rows);
    if cols == 0 || rows == 0 || ramp.is_empty() {
        return;
    }
    let span = (rows as i32 + LEAD_IN * 2) as f32;

    for x in 0..cols {
        // Per-column phase so the rain is ragged, not a flat sheet.
        let head = ((t * SPEED + x as f32 * 1.7) % span) as i32 - LEAD_IN;
        for y in 0..rows {
            let i = y * cols + x;
            let behind = head - y as i32;
            st.current[i] = if behind >= TRAIL {
                st.target[i]
            } else if behind >= 0 {
                ramp[rng.next_usize(ramp.len())]
            } else {
                ' '
            };
        }
    }
}
