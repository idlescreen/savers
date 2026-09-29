use crate::chaos::Chaos;
use crate::runner::Screensaver;
use crate::runner::TerminalCell;
use std::time::{Duration, Instant};

#[test]
fn test_performance_100_frames() {
    let mut chaos = Chaos::new();
    // Prevent slow sys_info calls in performance tests
    chaos.sys_refresh_timer = -1000.0;

    let cols = 120;
    let rows = 40;
    let mut grid = vec![TerminalCell::default(); cols * rows];

    // Force size initialization inside update
    chaos.update(Duration::from_millis(16), cols, rows);

    let start = Instant::now();

    for _ in 0..100 {
        chaos.update(Duration::from_millis(16), cols, rows);
        chaos.draw(&mut grid, cols, rows);
    }

    let elapsed = start.elapsed();
    println!("Performance test: 100 frames took {:?}", elapsed);

    // Assert it completes within a budget of 1500ms
    assert!(
        elapsed < Duration::from_millis(1500),
        "Performance test exceeded budget: {:?}",
        elapsed
    );
}
