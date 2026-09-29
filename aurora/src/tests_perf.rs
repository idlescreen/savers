use crate::aurora::Aurora;
use crate::runner::Screensaver;
use crate::runner::TerminalCell;
use std::time::{Duration, Instant};

#[test]
fn test_performance_aurora() {
    let mut aurora = Aurora::new();
    // Prevent slow system info calls by setting the refresh timer to a large negative value
    aurora.sys_refresh_timer = -1000.0;

    let cols = 80;
    let rows = 24;
    let mut grid = vec![TerminalCell::default(); cols * rows];
    let dt = Duration::from_millis(16);

    let start = Instant::now();

    for _ in 0..100 {
        aurora.update(dt, cols, rows);
        aurora.draw(&mut grid, cols, rows);
    }

    let elapsed = start.elapsed();
    println!(
        "Aurora performance test (100 frames) completed in: {:?}",
        elapsed
    );

    // Assert it completes within a reasonable budget (e.g. 1500ms)
    assert!(
        elapsed.as_millis() < 1500,
        "Performance test took too long: {:?}",
        elapsed
    );
}
