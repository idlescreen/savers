// SPDX-License-Identifier: MIT
// perf: T3 · metric: bounded single-pass work; no syscalls, no locks, no allocation on the steady path · check: review

//! Bench harness for the radar saver's per-tick hot path.
//!
//! Mirrors `cosmos/benches/tick.rs`, `gnats/benches/tick.rs`, and
//! `bursts/benches/tick.rs`. Radar runs enemy/defender/jet/laser
//! physics + sweep-angle + shield state; the bench pins
//! `sys_refresh_timer = -1000.0` and runs 50 warm-up ticks to
//! populate enemy + jet vecs before measuring.
//!
//! Run with:
//!
//! ```bash
//! cargo bench --bench tick
//! ```

use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use screensaver_radar::bench_exports::Radar;
use screensaver_radar::runner::{Screensaver, TerminalCell};

const GRIDS: &[(usize, usize)] = &[(80, 24), (160, 48), (210, 57)];

fn make_warm_radar(cols: usize, rows: usize) -> Radar {
    let mut effect = Radar::new();
    effect.prepare_for_bench(cols, rows);
    let mut grid = vec![TerminalCell::default(); cols * rows];
    // 50 ticks is enough for the spawn cadence to populate enemies.
    for _ in 0..50 {
        effect.update(Duration::from_millis(16), cols, rows);
        effect.draw(&mut grid, cols, rows);
    }
    effect
}

fn bench_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("radar_update");
    group.measurement_time(Duration::from_secs(3));
    group.warm_up_time(Duration::from_secs(1));

    for &(cols, rows) in GRIDS {
        let mut effect = make_warm_radar(cols, rows);
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
    let mut group = c.benchmark_group("radar_draw");
    group.measurement_time(Duration::from_secs(3));
    group.warm_up_time(Duration::from_secs(1));

    for &(cols, rows) in GRIDS {
        let effect = make_warm_radar(cols, rows);
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
