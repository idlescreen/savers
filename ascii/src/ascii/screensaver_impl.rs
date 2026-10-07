// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use super::Ascii;
use super::draw;
use super::effect;
use super::{BATTERY_SLOWDOWN, MAX_DT};
use crate::runner::Screensaver;
use crate::runner::TerminalCell;
use crate::runner::{get_system_info, query_current_palette};
use std::time::Duration;

impl Screensaver for Ascii {
    fn init(&mut self, cols: usize, rows: usize) {
        self.last_cols = cols;
        self.last_rows = rows;
        self.cells.resize(cols, rows);
        self.reload_art();
        self.dwell_left = self.effect.dwell();
    }

    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let speed = self.params.speed;
        let step = (dt.as_secs_f32().min(MAX_DT)) * speed;

        if cols != self.last_cols || rows != self.last_rows {
            self.last_cols = cols;
            self.last_rows = rows;
            self.cells.resize(cols, rows);
            self.reload_art();
            self.dwell_left = self.effect.dwell();
        }

        // Nothing resolved to draw — skip the per-cell pass rather than walk a
        // blank grid sixty times a second.
        if self.cells.inked_cells() == 0 {
            return;
        }

        // Power-adaptive: slow the animation on battery rather than blank.
        let rate = if self.on_battery {
            BATTERY_SLOWDOWN
        } else {
            1.0
        };
        let step = step * rate;

        if !self.pinned {
            self.dwell_left -= step;
            if self.dwell_left <= 0.0 {
                self.cycle_effect();
            }
        }

        let elapsed = self.dwell_left - self.effect.dwell();
        effect::advance(
            self.effect,
            &mut self.cells,
            step,
            elapsed.abs(),
            &self.ramp,
            &mut self.rng,
        );
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        let _ = (cols, rows);
        draw::paint(&self.cells, grid, self.fg, (0, 0, 0));
    }

    fn update_frame_time(&mut self, _dt: Duration) {
        // Refresh the power hint occasionally; the animation itself is a fixed
        // grid pass and has no adaptive quality dial.
        let sys = get_system_info();
        self.on_battery = sys.power_status.contains("Battery");
        self.fg = match self.params.fg {
            Some(rgb) => rgb,
            None => query_current_palette().accent,
        };
    }
}
