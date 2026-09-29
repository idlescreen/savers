use crate::ripple::Ripple;
use crate::runner::Screensaver;
use crate::runner::TerminalCell;
use std::time::{Duration, Instant};

#[test]
fn test_performance_ripple() {
    let mut fx = Ripple::new();
    let cols = 80;
    let rows = 24;
    let mut grid = vec![TerminalCell::default(); cols * rows];
    let dt = Duration::from_millis(16);
    let start = Instant::now();
    for _ in 0..100 {
        fx.update(dt, cols, rows);
        fx.draw(&mut grid, cols, rows);
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(1500),
        "Performance test exceeded budget: {:?}",
        elapsed
    );
}
