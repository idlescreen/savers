use crate::glyphs::Glyphs;
use crate::runner::Screensaver;
use crate::runner::TerminalCell;
use std::time::{Duration, Instant};

#[test]
fn test_screensaver_performance() {
    let mut glyphs = Glyphs::new();
    // Prevent slow system info calls by setting sys_refresh_timer to a large negative number
    glyphs.sys_refresh_timer = -1000.0;

    let cols = 80;
    let rows = 24;
    let mut grid = vec![TerminalCell::default(); cols * rows];

    // Initialize
    glyphs.init(cols, rows);

    let start = Instant::now();
    for _ in 0..100 {
        glyphs.update(Duration::from_millis(16), cols, rows);
        glyphs.draw(&mut grid, cols, rows);
    }
    let duration = start.elapsed();

    println!("Completed 100 frames in {:?}", duration);
    // Assert it completes within budget (e.g. 1500ms)
    assert!(
        duration < Duration::from_millis(1500),
        "Performance test exceeded 1500ms budget: {:?}",
        duration
    );
}
