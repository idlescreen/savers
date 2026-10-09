// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Consolidated ASCII logo screensaver effect module.
//!
//! **Taxonomy Classification**: Presentation Role (Purpose - Ambient Display).

pub mod cell_state;
pub mod draw;
pub mod effect;
pub mod load_art;
pub mod params;
pub mod ramp;

pub(crate) mod screensaver_impl;

use crate::runner::{LcgRng, get_system_info, query_current_palette};
use cell_state::CellState;
use effect::EffectKind;
use params::Params;

/// Frame delta clamp. A long stall must not teleport the animation.
const MAX_DT: f32 = 0.1;
/// Speed multiplier while on battery power, per the power-adaptive rule in
/// `AGENTS.md`.
const BATTERY_SLOWDOWN: f32 = 0.55;

/// Duration of the smooth visual transition in seconds between completed text and the incoming effect.
pub const TRANSITION_SECS: f32 = 0.7;

pub struct Ascii {
    pub(crate) rng: LcgRng,
    pub(crate) playlist: effect::EffectPlaylist,
    pub(crate) cells: CellState,
    pub(crate) params: Params,
    pub(crate) effect: EffectKind,
    /// True when `[saver] ascii.effect` pins one effect instead of rotating.
    pub(crate) pinned: bool,
    pub(crate) dwell_left: f32,
    pub(crate) fg: (u8, u8, u8),
    pub(crate) on_battery: bool,
    pub(crate) last_cols: usize,
    pub(crate) last_rows: usize,
    pub(crate) art_text: String,
    pub(crate) current_frame: String,
    pub(crate) engine: Option<screensaver_impl::EngineSession>,
    pub(crate) brand_target: load_art::BrandTarget,
    pub(crate) frame_grid: Vec<crate::runner::TerminalCell>,
    pub(crate) transition_active: bool,
    pub(crate) transition_left: f32,
    pub(crate) transition_duration: f32,
    pub(crate) transition_from_grid: Vec<crate::runner::TerminalCell>,
}

impl Default for Ascii {
    fn default() -> Self {
        Self::new()
    }
}

impl Ascii {
    pub fn new() -> Self {
        let params = Params::read();
        let pinned = params.effect.is_some();
        let mut rng = LcgRng::from_env_or_random();
        let mut playlist = effect::EffectPlaylist::new();
        let effect = params
            .effect
            .unwrap_or_else(|| playlist.next_effect(&mut rng));
        let dwell_left = params.cycle_secs.unwrap_or_else(|| effect.dwell());
        let brand_target = load_art::BrandTarget::random_excluding(&mut rng, None);

        Self {
            rng,
            playlist,
            cells: CellState::new(),
            params,
            effect,
            pinned,
            dwell_left,
            fg: (248, 248, 242),
            on_battery: get_system_info().power_status.contains("Battery"),
            last_cols: 0,
            last_rows: 0,
            art_text: String::new(),
            current_frame: String::new(),
            engine: None,
            brand_target,
            frame_grid: Vec::new(),
            transition_active: false,
            transition_left: 0.0,
            transition_duration: TRANSITION_SECS,
            transition_from_grid: Vec::new(),
        }
        .with_theme_fg()
    }

    /// Effective dwell duration in seconds for the active effect.
    pub fn dwell_time(&self) -> f32 {
        self.params
            .cycle_secs
            .unwrap_or_else(|| self.effect.dwell())
    }

    /// Adopt the session's accent colour unless `[saver] ascii.fg` overrides it.
    fn with_theme_fg(mut self) -> Self {
        self.fg = match self.params.fg {
            Some(rgb) => rgb,
            None => query_current_palette().accent,
        };
        self
    }

    /// Resolve the art into the grid, restarting the current effect.
    pub fn reload_art(&mut self) {
        self.art_text =
            load_art::resolve_art_for_target(self.brand_target, self.last_cols, self.last_rows);
        let lines =
            load_art::resolve_lines_for_target(self.brand_target, self.last_cols, self.last_rows);
        self.cells.load(&lines);
    }

    /// Start or restart the underlying ttfx animation session.
    pub(crate) fn start_engine_session(&mut self) {
        self.current_frame.clear();
        self.frame_grid.fill(crate::runner::TerminalCell::default());
        if self.last_cols == 0 || self.last_rows == 0 || self.art_text.trim().is_empty() {
            self.engine = None;
            return;
        }
        let seed = self.rng.next_u64();
        self.engine = screensaver_impl::EngineSession::new(
            &self.art_text,
            self.effect,
            self.last_cols,
            self.last_rows,
            seed,
        );
        if let Some(frame) = self
            .engine
            .as_mut()
            .and_then(|session| session.next_frame())
        {
            self.current_frame.push_str(&frame);
            draw::paint_frame(
                &self.current_frame,
                &mut self.frame_grid,
                self.last_cols,
                self.last_rows,
                self.fg,
                (0, 0, 0),
            );
        }
    }

    /// Smoothly transitions from the current settled frame into the next randomized target and effect.
    pub fn begin_transition(&mut self) {
        if self.frame_grid.is_empty() {
            self.cycle_effect();
            return;
        }
        self.transition_from_grid = self.frame_grid.clone();
        let dur = TRANSITION_SECS.min(self.dwell_time() * 0.4);
        self.transition_duration = dur;
        self.transition_left = dur;
        self.transition_active = true;
        if !self.pinned {
            self.cycle_effect();
        } else {
            self.cycle_target();
        }
    }

    /// Advance brand target randomly without immediate repeats, pick next effect, and restart progress.
    pub fn cycle_effect(&mut self) {
        self.brand_target =
            load_art::BrandTarget::random_excluding(&mut self.rng, Some(self.brand_target));
        self.reload_art();
        self.effect = self.playlist.next_effect(&mut self.rng);
        self.dwell_left = self.dwell_time();
        self.cells.settled.fill(0.0);
        self.cells.progress = 0.0;
        self.start_engine_session();
    }

    /// Advance brand target randomly without immediate repeats while keeping a pinned effect.
    pub fn cycle_target(&mut self) {
        self.brand_target =
            load_art::BrandTarget::random_excluding(&mut self.rng, Some(self.brand_target));
        self.reload_art();
        self.dwell_left = self.dwell_time();
        self.cells.settled.fill(0.0);
        self.cells.progress = 0.0;
        self.start_engine_session();
    }

    /// Exposed for benches and tests that need a settled, representative state.
    pub fn prepare_for_bench(&mut self, cols: usize, rows: usize) {
        self.last_cols = cols;
        self.last_rows = rows;
        self.frame_grid
            .resize(cols * rows, crate::runner::TerminalCell::default());
        self.cells.resize(cols, rows);
        self.reload_art();
        self.start_engine_session();
    }

    /// Short description of the live effect, for diagnostics and previews.
    pub fn label(&self) -> String {
        let mode = if self.pinned { "pinned" } else { "random" };
        format!("{} · {mode}", self.effect.name())
    }

    /// Pin the active effect, stopping `random` rotation.
    pub fn pin_effect(&mut self, kind: EffectKind) {
        self.effect = kind;
        self.pinned = true;
        self.dwell_left = self.dwell_time();
        self.cells.settled.fill(0.0);
        self.cells.progress = 0.0;
        self.playlist.set_last(kind);
        self.start_engine_session();
    }
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
