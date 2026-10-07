// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use crate::ascii::Ascii;
use crate::runner::{Screensaver, TerminalCell};
use std::time::{Duration, Instant};

#[test]
fn test_screensaver_performance() {
    let mut saver = Ascii::new();
    let cols = 80;
    let rows = 24;
    let mut grid = vec![TerminalCell::default(); cols * rows];
    saver.init(cols, rows);

    let start = Instant::now();
    for _ in 0..100 {
        saver.update(Duration::from_millis(16), cols, rows);
        saver.draw(&mut grid, cols, rows);
    }
    let duration = start.elapsed();

    println!("Completed 100 frames in {duration:?}");
    assert!(
        duration < Duration::from_millis(1500),
        "Performance test exceeded 1500ms budget: {duration:?}"
    );
}

#[test]
fn render_loop_stays_within_the_frame_budget() {
    let mut saver = Ascii::new();
    let cols = 200;
    let rows = 60;
    let mut grid = vec![TerminalCell::default(); cols * rows];
    saver.init(cols, rows);

    let start = Instant::now();
    for _ in 0..60 {
        saver.update(Duration::from_millis(16), cols, rows);
        saver.draw(&mut grid, cols, rows);
    }
    let per_frame = start.elapsed() / 60;
    assert!(
        per_frame < Duration::from_millis(16),
        "frame took {per_frame:?}, over the 16.6ms budget"
    );
}