// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! WebAssembly host container for screensaver rendering.

use idle_api::{ScreensaverInstance, TerminalCell};
use std::time::Duration;

/// Owns the saver instance plus its render grid and packed output buffer.
pub struct SaverHost {
    pub(crate) inst: Box<ScreensaverInstance>,
    pub(crate) grid: Vec<TerminalCell>,
    pub(crate) packed: Vec<u32>,
    pub(crate) cols: usize,
    pub(crate) rows: usize,
}

impl SaverHost {
    pub fn new(cols: usize, rows: usize) -> Option<Box<Self>> {
        if cols == 0 || rows == 0 || cols > 512 || rows > 256 {
            return None;
        }
        let raw = screensaver_beams::create_screensaver();
        if raw.is_null() {
            return None;
        }
        let mut inst = unsafe { Box::from_raw(raw) };
        inst.inner.init(cols, rows);
        inst.inner.set_active(true);
        inst.inner.set_focused(true);
        Some(Box::new(SaverHost {
            inst,
            grid: vec![TerminalCell::default(); cols * rows],
            packed: vec![0u32; cols * rows * 3],
            cols,
            rows,
        }))
    }

    pub fn tick(&mut self, dt_ms: f64) -> *const u32 {
        let dt = Duration::from_secs_f64((dt_ms / 1000.0).max(0.0));
        self.inst.inner.update(dt, self.cols, self.rows);
        self.inst.inner.update_frame_time(dt);
        self.inst.inner.draw(&mut self.grid, self.cols, self.rows);
        for (i, c) in self.grid.iter().enumerate() {
            self.packed[i * 3] = c.ch as u32;
            self.packed[i * 3 + 1] = (c.fg.0 as u32)
                | (c.fg.1 as u32) << 8
                | (c.fg.2 as u32) << 16
                | (u32::from(c.bold) << 24);
            self.packed[i * 3 + 2] = (c.bg.0 as u32) | (c.bg.1 as u32) << 8 | (c.bg.2 as u32) << 16;
        }
        self.packed.as_ptr()
    }

    pub fn cells_len(&self) -> usize {
        self.packed.len()
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        if cols == 0 || rows == 0 || cols > 512 || rows > 256 {
            return;
        }
        self.cols = cols;
        self.rows = rows;
        self.grid = vec![TerminalCell::default(); cols * rows];
        self.packed = vec![0u32; cols * rows * 3];
        self.inst.inner.init(cols, rows);
    }
}
