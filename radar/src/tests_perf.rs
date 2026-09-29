// SPDX-License-Identifier: MIT

use crate::radar::Radar;
use crate::runner::Screensaver;
use crate::runner::TerminalCell;
use std::time::{Duration, Instant};

#[test]
fn test_performance_benchmark() {
    let mut radar = Radar::new();

    let cols = 80;
    let rows = 24;
    radar.init(cols, rows);

    let mut grid = vec![TerminalCell::default(); cols * rows];
    let frame_dt = Duration::from_millis(16);

    let start = Instant::now();

    for _ in 0..100 {
        radar.update(frame_dt, cols, rows);
        radar.draw(&mut grid, cols, rows);
    }

    let elapsed = start.elapsed();
    println!("100 frames completed in: {:?}", elapsed);

    let budget = if cfg!(debug_assertions) {
        Duration::from_millis(5000)
    } else {
        Duration::from_millis(1500)
    };

    assert!(
        elapsed < budget,
        "Performance budget exceeded: {:?}",
        elapsed
    );
}
