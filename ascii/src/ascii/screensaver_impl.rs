// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use super::Ascii;
use super::draw;
use super::effect::EffectKind;
use super::{BATTERY_SLOWDOWN, MAX_DT};
use crate::runner::Screensaver;
use crate::runner::TerminalCell;
use crate::runner::{get_system_info, query_current_palette};
use clap::Parser;
use std::time::Duration;
use ttfx::cli::Cli;
use ttfx::engine::canvas::Anchor;
use ttfx::engine::ctx::{Clock, EngineCtx};
use ttfx::engine::effect::Effect;

pub(crate) struct EngineSession {
    ctx: EngineCtx,
    effect: Box<dyn Effect>,
}

impl EngineSession {
    pub(crate) fn new(
        art: &str,
        kind: EffectKind,
        cols: usize,
        rows: usize,
        seed: u64,
    ) -> Option<Self> {
        if cols == 0 || rows == 0 || art.trim().is_empty() {
            return None;
        }
        let cli = Cli::try_parse_from(["ttfx", kind.name()]).ok()?;
        let mut config = cli.terminal_config();
        config.canvas_width = cols as i64;
        config.canvas_height = rows as i64;
        config.ignore_terminal_dimensions = true;
        config.anchor_text = Anchor::C;

        let clock = Clock::virtual_with_frame_rate(60);
        let rng = ttfx::utils::rng::Rng::seeded(seed);
        let mut ctx = EngineCtx::new(art, config, rng, clock).ok()?;
        let mut effect = cli.effect?.build_effect();
        effect.build(&mut ctx).ok()?;

        Some(Self { ctx, effect })
    }

    pub(crate) fn next_frame(&mut self) -> Option<String> {
        self.effect.next_frame(&mut self.ctx)
    }
}

impl Screensaver for Ascii {
    fn init(&mut self, cols: usize, rows: usize) {
        self.last_cols = cols;
        self.last_rows = rows;
        self.cells.resize(cols, rows);
        self.frame_grid.resize(cols * rows, TerminalCell::default());
        self.frame_grid.fill(TerminalCell::default());
        self.transition_from_grid.clear();
        self.transition_active = false;
        self.transition_left = 0.0;
        self.playlist = super::effect::EffectPlaylist::new_with_last(Some(self.effect));
        self.brand_target = super::load_art::BrandTarget::random_excluding(&mut self.rng, None);
        self.reload_art();
        self.dwell_left = self.dwell_time();
        self.start_engine_session();
    }

    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let speed = self.params.speed;
        let step = (dt.as_secs_f32().min(MAX_DT)) * speed;

        if cols != self.last_cols || rows != self.last_rows {
            self.last_cols = cols;
            self.last_rows = rows;
            self.cells.resize(cols, rows);
            self.frame_grid.resize(cols * rows, TerminalCell::default());
            self.frame_grid.fill(TerminalCell::default());
            self.transition_from_grid.clear();
            self.transition_active = false;
            self.transition_left = 0.0;
            self.reload_art();
            self.dwell_left = self.dwell_time();
            self.start_engine_session();
        }

        // Power-adaptive: slow the animation on battery rather than blank.
        let rate = if self.on_battery {
            BATTERY_SLOWDOWN
        } else {
            1.0
        };
        let step = step * rate;

        if self.transition_active {
            self.transition_left -= step;
            if self.transition_left <= 0.0 {
                self.transition_active = false;
                self.transition_from_grid.clear();
            }
        }

        self.dwell_left -= step;
        if self.dwell_left <= 0.0 && !self.transition_active {
            self.begin_transition();
            return;
        }

        if let Some(ref mut session) = self.engine {
            match session.next_frame() {
                Some(frame) => {
                    self.current_frame.clear();
                    self.current_frame.push_str(&frame);
                    draw::paint_frame(
                        &self.current_frame,
                        &mut self.frame_grid,
                        cols,
                        rows,
                        self.fg,
                        (0, 0, 0),
                    );
                }
                None => {
                    // Animation complete: drop the engine session and dwell on
                    // the settled frame until dwell_left expires.
                    self.engine = None;
                }
            }
        }
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        if self.art_text.trim().is_empty() {
            grid.fill(TerminalCell::default());
            return;
        }
        if self.transition_active
            && !self.transition_from_grid.is_empty()
            && self.transition_from_grid.len() == grid.len()
            && self.frame_grid.len() == grid.len()
        {
            let t = if self.transition_duration > 0.0 {
                (1.0 - (self.transition_left / self.transition_duration)).clamp(0.0, 1.0)
            } else {
                1.0
            };
            draw::paint_transition(
                &self.transition_from_grid,
                &self.frame_grid,
                grid,
                t,
                cols,
                rows,
            );
            return;
        }
        if !self.frame_grid.is_empty() {
            let n = self.frame_grid.len().min(grid.len());
            grid[..n].copy_from_slice(&self.frame_grid[..n]);
            if grid.len() > n {
                grid[n..].fill(TerminalCell::default());
            }
        } else if !self.current_frame.is_empty() {
            draw::paint_frame(&self.current_frame, grid, cols, rows, self.fg, (0, 0, 0));
        } else if self.cells.inked_cells() > 0 {
            draw::paint(&self.cells, grid, self.fg, (0, 0, 0));
        } else {
            grid.fill(TerminalCell::default());
        }
    }

    fn update_frame_time(&mut self, _dt: Duration) {
        let sys = get_system_info();
        self.on_battery = sys.power_status.contains("Battery");
        self.fg = match self.params.fg {
            Some(rgb) => rgb,
            None => query_current_palette().accent,
        };
    }
}
