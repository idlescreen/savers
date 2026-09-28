// SPDX-License-Identifier: MIT
// perf: T3 · metric: bounded single-pass work; no syscalls, no locks, no allocation on the steady path · check: review

//! Bench harness for the cosmos saver's per-tick hot path.
//!
//! Two workloads:
//!
//! - `update_*` — [`Cosmos::update`] (via the [`Screensaver`] trait).
//!   Drives the universe state machine and the particle physics. Runs
//!   100 warm-up ticks to settle into the `Accretion` state (where the
//!   particle budget is at its 580-particle ceiling) before measuring,
//!   so the bench measures steady-state cost rather than ramp-up cost.
//! - `draw_*` — [`Cosmos::draw`]. Reads the current state and writes a
//!   `TerminalCell` grid. This is what the runtime hands to
//!   `CellRenderer` for upscale.
//!
//! Parametrized over realistic grid sizes:
//!
//! - 80×24 (VT100 default — preview / small window path)
//! - 160×48 (typical 2× presentation)
//! - 210×57 (the daemon span-grid cap, see `test_span_grid_perf_benchmark`)
//!
//! Every input is wrapped in `std::hint::black_box` per plan §9 (bench
//! harness correctness risk) so the compiler can't constant-fold a
//! known-`Cosmos::new()` (which has zero particles) and report "0
//! cycles".
//!
//! Run with:
//!
//! ```bash
//! cargo bench --bench tick
//! # one bench:
//! cargo bench --bench tick -- "update_210x57"
//! ```

use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use screensaver_cosmos::bench_exports::Cosmos;
use screensaver_cosmos::runner::{Screensaver, TerminalCell};

/// Realistic grid sizes. 210×57 is the daemon's span-grid cap
/// (`test_span_grid_perf_benchmark`).
const GRIDS: &[(usize, usize)] = &[(80, 24), (160, 48), (210, 57)];

/// Frame interval we feed to `update()`. 16 ms = 60 Hz; the cosmos
/// saver runs at the daemon's `tick_hz` (default 60 Hz).
const FRAME_DT: Duration = Duration::from_millis(16);

/// Build a `Cosmos` instance warmed up into the `Accretion` state with
/// a fully-populated particle field. `Cosmos::prepare_for_bench` does
/// the state pinning; the warm-up loop then populates the particle
/// buffer via `update()`. After 100 ticks the state is in steady-state
/// Accretion with the 580-particle budget full.
fn make_warm_cosmos(cols: usize, rows: usize) -> Cosmos {
    let mut effect = Cosmos::new();
    effect.prepare_for_bench(cols, rows);
    let mut grid = vec![TerminalCell::default(); cols * rows];
    for _ in 0..100 {
        effect.update(FRAME_DT, cols, rows);
        effect.draw(&mut grid, cols, rows);
    }
    effect
}

fn bench_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("cosmos_update");
    group.measurement_time(Duration::from_secs(3));
    group.warm_up_time(Duration::from_secs(1));

    for &(cols, rows) in GRIDS {
        let mut effect = make_warm_cosmos(cols, rows);
        let label = format!("update_{cols}x{rows}");
        group.bench_with_input(
            BenchmarkId::from_parameter(&label),
            &(cols, rows),
            |b, &(cols, rows)| {
                b.iter(|| {
                    effect.update(black_box(FRAME_DT), black_box(cols), black_box(rows));
                    black_box(&mut effect as *mut _);
                });
            },
        );
    }
    group.finish();
}

fn bench_draw(c: &mut Criterion) {
    let mut group = c.benchmark_group("cosmos_draw");
    group.measurement_time(Duration::from_secs(3));
    group.warm_up_time(Duration::from_secs(1));

    for &(cols, rows) in GRIDS {
        let effect = make_warm_cosmos(cols, rows);
        let mut grid = vec![TerminalCell::default(); cols * rows];
        let label = format!("draw_{cols}x{rows}");
        group.bench_with_input(
            BenchmarkId::from_parameter(&label),
            &(cols, rows),
            |b, &(cols, rows)| {
                b.iter(|| {
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
