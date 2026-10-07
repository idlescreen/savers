// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Terminal preview for the `ascii` saver.
//!
//! Drives the real `Ascii` screensaver off-screen and paints each cell with
//! 24-bit ANSI colour, so the animation can be inspected without a compositor.
//! This is the quickest way to judge an effect's pacing and legibility.
//!
//! ```sh
//! cargo run -p ascii --example showcase -- random IDLE
//! cargo run -p ascii --example showcase -- decrypt IDLE
//! cargo run -p ascii --example showcase -- wave OMARCHY
//! ```

use idle_api::{saver_param_env_key, set_env};
use screensaver_ascii::bench_exports::Ascii;
use screensaver_ascii::runner::{Screensaver, TerminalCell};
use std::io::{self, Write};
use std::time::Duration;

const COLS: usize = 96;
const ROWS: usize = 28;
/// Simulation steps between painted frames (~0.2s of animation).
const STRIDE: usize = 12;
/// Painted frames to emit.
const FRAMES: usize = 24;

fn main() {
    let mut args = std::env::args().skip(1);
    let effect = args.next().unwrap_or_else(|| "random".into());
    let text = args.next().unwrap_or_else(|| "IDLE".into());
    let ramp = args.next().unwrap_or_else(|| "blocks".into());

    // `set_env` is the host's own wrapper around the unsafe env write, so the
    // example stays free of `unsafe` blocks.
    let params = [("effect", effect.as_str()), ("text", text.as_str()), ("ramp", ramp.as_str())];
    for (key, value) in params {
        if let Some(name) = saver_param_env_key(key) {
            set_env(&name, value);
        }
    }

    let mut saver = Ascii::new();
    saver.init(COLS, ROWS);

    let mut grid = vec![TerminalCell::default(); COLS * ROWS];
    let mut out = io::stdout().lock();

    for frame in 0..FRAMES * STRIDE {
        saver.update(Duration::from_millis(16), COLS, ROWS);
        if frame % STRIDE != 0 {
            continue;
        }
        saver.draw(&mut grid, COLS, ROWS);
        let _ = writeln!(out, "\x1b[90m── {} ─────────────────────────\x1b[0m", saver.label());
        let _ = write!(out, "{}", paint(&grid));
        let _ = out.flush();
        std::thread::sleep(Duration::from_millis(40));
    }
}

/// Paint a cell grid as truecolor ANSI text. Allocates freely — this is a
/// developer tool, not the render loop.
fn paint(grid: &[TerminalCell]) -> String {
    let mut out = String::with_capacity(COLS * ROWS * 14);
    for y in 0..ROWS {
        for x in 0..COLS {
            let cell = &grid[y * COLS + x];
            if cell.ch == ' ' {
                out.push(' ');
                continue;
            }
            let (r, g, b) = cell.fg;
            out.push_str(&format!("\x1b[38;2;{r};{g};{b}m{}\x1b[0m", cell.ch));
        }
        out.push('\n');
    }
    out
}