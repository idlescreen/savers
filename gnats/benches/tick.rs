// SPDX-License-Identifier: MIT
// perf: T3 · metric: bounded single-pass work; no syscalls, no locks, no allocation on the steady path · check: review

//! Bench harness for the gnats saver's per-tick hot path.
//!
//! Mirrors `cosmos/benches/tick.rs`. Two workloads:
//!
//! - `update_<cols>x<rows>` — [`Screensaver::update`]. Drives the
//!   firefly physics + predator-breath dynamics + per-cell resize
//!   handling. The bench pre-pins `last_cols`/`last_rows` so the
//!   resize-init branch (which spawns attractors + clears particle
//!   vecs) doesn't fire on every iteration.
//! - `draw_<cols>x<rows>` — [`Screensaver::draw`]. Writes a
//!   `TerminalCell` grid from the current particle state.
//!
//! Parametrized over realistic grid sizes (same set as cosmos):
//! 80×24, 160×48, 210×57. Every input is wrapped in
//! `std::hint::black_box` per plan §9.
//!
//! Run with:
//!
//! ```bash
//! cargo bench --bench tick
//! ```

use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use screensaver_gnats::bench_exports::Gnats;
use screensaver_gnats::runner::{Screensaver, TerminalCell};

const GRIDS: &[(usize, usize)] = &[(80, 24), (160, 48), (210, 57)];

/// Gnats is a `pub fn new()` + state machine; `prepare_for_bench`
/// skips the slow system-info probe by setting
/// `sys_refresh_timer = -1000.0`. We also pre-allocate the particle
/// vectors by triggering one warm-up update; otherwise
/// `Vec::push()` overhead from cold-start confounds the measurement.
fn make_warm_gnats(cols: usize, rows: usize) -> Gnats {
    let mut g = Gnats::new();
    g.prepare_for_bench(cols, rows);
    let mut grid = vec![TerminalCell::default(); cols * rows];
    // 50 ticks is enough for `adjust_populations` to populate the
    // firefly / attractor vecs to steady state.
    for _ in 0..50 {
        g.update(Duration::from_millis(16), cols, rows);
        g.draw(&mut grid, cols, rows);
    }
    g
}

fn bench_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("gnats_update");
    group.measurement_time(Duration::from_secs(3));
    group.warm_up_time(Duration::from_secs(1));

    for &(cols, rows) in GRIDS {
        let mut g = make_warm_gnats(cols, rows);
        let label = format!("update_{cols}x{rows}");
        group.bench_with_input(
            BenchmarkId::from_parameter(&label),
            &(cols, rows),
            |b, &(cols, rows)| {
                b.iter(|| {
                    g.update(
                        black_box(Duration::from_millis(16)),
                        black_box(cols),
                        black_box(rows),
                    );
                    black_box(&mut g as *mut _);
                });
            },
        );
    }
    group.finish();
}

fn bench_draw(c: &mut Criterion) {
    let mut group = c.benchmark_group("gnats_draw");
    group.measurement_time(Duration::from_secs(3));
    group.warm_up_time(Duration::from_secs(1));

    for &(cols, rows) in GRIDS {
        let g = make_warm_gnats(cols, rows);
        let mut grid = vec![TerminalCell::default(); cols * rows];
        let label = format!("draw_{cols}x{rows}");
        group.bench_with_input(
            BenchmarkId::from_parameter(&label),
            &(cols, rows),
            |b, &(cols, rows)| {
                b.iter(|| {
                    g.draw(black_box(&mut grid), black_box(cols), black_box(rows));
                    black_box(&grid as *const _);
                    black_box(grid.len());
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_update, bench_draw);
criterion_main!(benches);
