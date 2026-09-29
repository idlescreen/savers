// SPDX-License-Identifier: MIT

//! Property stress test: hostile `dt` and grid dimensions through
//! the shared harness.
//!
//! The harness (`idle_api::stress::stress_saver`) feeds the
//! saver a stream of randomised frame times and surface sizes
//! designed to break three classes of bugs:
//!
//!   1. **NaN propagation** — a `dt` of 0 or an extreme value must
//!      not poison the saver's accumulator fields (the saver
//!      survives `dt` clamping to a sane range; this test catches
//!      a regression that drops the clamp).
//!   2. **Resize churn** — flipping cols/rows on every iteration
//!      stresses the init path; a saver that forgets to clear its
//!      scratch buffers between resizes will surface as out-of-
//!      bounds writes here.
//!   3. **Deterministic seed** — `stress_saver` uses a fixed seed
//!      (`0xC0FFEE`) so the run is reproducible across CI and
//!      developer laptops; the magic constant is the project's
//!      "kitchen-sink" seed.
//!
//! If this test regresses, the saver likely needs a closer look
//! at its `init`, `update`, or `draw_frame` paths before
//! shipping.
//!
//! The saver type lives under `crate::bench_exports` because the
//! cdylib's only public surface is the `ScreensaverInstance` FFI;
//! `bench_exports` is a `#[doc(hidden)]` escape hatch that the
//! bench + this stress fixture consume.

use crate::bench_exports::Cosmos;
use idle_api::stress::stress_saver;

#[test]
fn stress_survives_hostile_frames() {
    let mut s = Cosmos::new();
    stress_saver(&mut s, 1000, 0xC0FFEE);
}
