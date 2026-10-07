// SPDX-License-Identifier: MIT

//! Bench harness for the ascii saver's per-tick hot path.
//!
//! Two workloads:
//!
//! - `update_<effect>_<cols>x<rows>` — [`Screensaver::update`]. Advances one
//!   pinned effect over the pre-allocated cell buffers.
//! - `draw_<cols>x<rows>` — [`Screensaver::draw`]. Writes the `TerminalCell`
//!   grid from the current cell state.
//!
//! Every effect is measured at every grid size. That matters here in a way it
//! does not for the particle savers: `Ascii::new()` picks an effect at random,
//! so an unpinned benchmark would compare different algorithms across sizes
//! and report nonsense. `pin_effect` removes that variance.
//!
//! Grids match cosmos: 80×24, 160×48, 210×57. `prepare_for_bench` skips the
//! art-resolution probe; 50 warm-up ticks settle state before measuring.
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
use screensaver_ascii::bench_exports::{Ascii, EffectKind};
use screensaver_ascii::runner::{Screensaver, TerminalCell};

const GRIDS: &[(usize, usize)] = &[(80, 24), (160, 48), (210, 57)];
const EFFECTS: [EffectKind; 6] = EffectKind::ALL;

fn make_warm_ascii(cols: usize, rows: usize, kind: EffectKind) -> Ascii {
    let mut effect = Ascii::new();
    effect.prepare_for_bench(cols, rows);
    effect.pin_effect(kind);
    let mut grid = vec![TerminalCell::default(); cols * rows];
    for _ in 0..50 {
        effect.update(Duration::from_millis(16), cols, rows);
        effect.draw(&mut grid, cols, rows);
    }
    effect
}

fn bench_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("ascii_update");
    group.measurement_time(Duration::from_secs(3));
    group.warm_up_time(Duration::from_secs(1));

    for kind in EFFECTS {
        for &(cols, rows) in GRIDS {
            let mut effect = make_warm_ascii(cols, rows, kind);
            let label = format!("{}_{cols}x{rows}", kind.name());
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
    }
    group.finish();
}

fn bench_draw(c: &mut Criterion) {
    let mut group = c.benchmark_group("ascii_draw");
    group.measurement_time(Duration::from_secs(3));
    group.warm_up_time(Duration::from_secs(1));

    for &(cols, rows) in GRIDS {
        let effect = make_warm_ascii(cols, rows, EFFECTS[0]);
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
