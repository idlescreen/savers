use crate::cosmos::{Cosmos, UniverseState};
use crate::runner::Screensaver;
use crate::runner::TerminalCell;
use std::time::{Duration, Instant};

#[test]
fn test_screensaver_perf_benchmark() {
    let mut effect = Cosmos::new();

    // Prevent slow system info queries in performance tests
    effect.sys_refresh_timer = -1000.0;

    let cols = 80;
    let rows = 24;
    let mut grid = vec![TerminalCell::default(); cols * rows];

    let start = Instant::now();

    // Simulate 100 frames
    for _ in 0..100 {
        effect.update(Duration::from_millis(16), cols, rows);
        effect.draw(&mut grid, cols, rows);
    }

    let duration = start.elapsed();
    println!(
        "Completed 100 frames of Cosmos screensaver in {:?}",
        duration
    );

    // Assert it completes within a budget of 1500ms
    assert!(
        duration < Duration::from_millis(1500),
        "Performance test exceeded budget: {:?}",
        duration
    );
}

#[test]
fn test_span_grid_perf_benchmark() {
    let mut effect = Cosmos::new();
    effect.sys_refresh_timer = -1000.0;
    effect.state = UniverseState::Accretion;
    effect.state_timer = 2.0;

    // Span grid size after daemon cap (210x57)
    let cols = 210;
    let rows = 57;
    effect.refresh_screen_cache(cols, rows);
    effect.last_cols = cols;
    effect.last_rows = rows;

    let mut grid = vec![TerminalCell::default(); cols * rows];

    let start = Instant::now();
    for _ in 0..100 {
        effect.update(Duration::from_millis(38), cols, rows);
        effect.draw(&mut grid, cols, rows);
    }
    let duration = start.elapsed();
    println!(
        "Completed 100 span-grid frames ({cols}x{rows}) in {:?}",
        duration
    );

    assert!(
        duration < Duration::from_millis(4000),
        "Span-grid performance test exceeded budget: {:?}",
        duration
    );
}
