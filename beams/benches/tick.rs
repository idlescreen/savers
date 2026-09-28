// SPDX-License-Identifier: MIT
// perf: T3 · metric: bounded single-pass work; no syscalls, no locks, no allocation on the steady path · check: review

//! Bench harness for the beams saver's per-tick hot path.
//!
//! Mirrors cosmos/gnats/bursts/radar. Two workloads:
//!
//! - `update_<cols>x<rows>` — [`Screensaver::update`]. Drives the
//!   beams per-tick state machine + physics.
//! - `draw_<cols>x<rows>` — [`Screensaver::draw`]. Writes a
//!   `TerminalCell` grid from the current state.
//!
//! Parametrized over the same 3 grid sizes as cosmos: 80×24, 160×48,
//! 210×57. `prepare_for_bench` skips the slow system-info probe;
//! 50 warm-up ticks populate any particle vecs before measuring.
//! Every input wrapped in `std::hint::black_box` per plan §9.
//!
//! Run with:
//!
//! ```bash
//! cargo bench --bench tick
//! ```

use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use screensaver_beams::bench_exports::Beams;
use screensaver_beams::runner::{Screensaver, TerminalCell};

const GRIDS: &[(usize, usize)] = &[(80, 24), (160, 48), (210, 57)];

fn make_warm_beams(cols: usize, rows: usize) -> Beams {
    let mut effect = Beams::new();
    effect.prepare_for_bench(cols, rows);
    let mut grid = vec![TerminalCell::default(); cols * rows];
    for _ in 0..50 {
        effect.update(Duration::from_millis(16), cols, rows);
        effect.draw(&mut grid, cols, rows);
    }
    effect
}

fn bench_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("beams_update");
    group.measurement_time(Duration::from_secs(3));
    group.warm_up_time(Duration::from_secs(1));

    for &(cols, rows) in GRIDS {
        let mut effect = make_warm_beams(cols, rows);
        let label = format!("update_{cols}x{rows}");
        group.bench_with_input(
            BenchmarkId::from_parameter(&label),
            &(cols, rows),
            |bencher, &(cols, rows)| {
                bencher.iter(|| {
                    effect.update(
                        black_box(Duration::from_millis(16)),
                        black_box(cols),
                        black_box(rows),
                    );
                    black_box(&mut effect as *mut _);
                });
            },
        );
    }
    group.finish();
}

fn bench_draw(c: &mut Criterion) {
    let mut group = c.benchmark_group("beams_draw");
    group.measurement_time(Duration::from_secs(3));
    group.warm_up_time(Duration::from_secs(1));

    for &(cols, rows) in GRIDS {
        let effect = make_warm_beams(cols, rows);
        let mut grid = vec![TerminalCell::default(); cols * rows];
        let label = format!("draw_{cols}x{rows}");
        group.bench_with_input(
            BenchmarkId::from_parameter(&label),
            &(cols, rows),
            |bencher, &(cols, rows)| {
                bencher.iter(|| {
                    effect.draw(black_box(&mut grid), black_box(cols), black_box(rows));
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
