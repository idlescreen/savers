// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use crate::ascii::Ascii;
use crate::runner::{Screensaver, TerminalCell};
use std::time::Duration;

/// Hammers resize, effect cycling, and degenerate grids. Nothing here may
/// panic — the host runs this saver unattended for hours.
#[test]
fn chaos_resize_and_cycle() {
    let mut saver = Ascii::new();
    saver.pinned = false;
    let mut grid = vec![TerminalCell::default(); 256 * 64];

    let dims = [(80, 24), (200, 60), (1, 1), (0, 0), (40, 120), (13, 7)];
    let mut step = 0usize;
    for round in 0..200 {
        let (cols, rows) = dims[round % dims.len()];
        saver.update(Duration::from_millis(16), cols, rows);
        if cols * rows <= grid.len() {
            saver.draw(&mut grid[..cols * rows], cols, rows);
        }
        step += 1;
    }
    assert!(step > 0);
}

#[test]
fn chaos_long_run_with_random_timing() {
    let mut saver = Ascii::new();
    saver.pinned = false;
    let (cols, rows) = (100, 40);
    let mut grid = vec![TerminalCell::default(); cols * rows];
    saver.init(cols, rows);

    // Alternating tiny and huge deltas must not desync or panic.
    for i in 0..300 {
        let dt = if i % 3 == 0 { 0 } else { 250 };
        saver.update(Duration::from_millis(dt), cols, rows);
        saver.draw(&mut grid, cols, rows);
    }
}
