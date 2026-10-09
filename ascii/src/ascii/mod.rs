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
    pub(crate) start_dwell: bool,
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
        let start_dwell = !pinned && load_art::custom_text().is_none();
        let dwell_left = if start_dwell {
            params.cycle_secs.unwrap_or(2.0)
        } else {
            params.cycle_secs.unwrap_or_else(|| effect.dwell())
        };

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
            brand_target: load_art::BrandTarget::Os,
            start_dwell,
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
            self.current_frame.clear();
            self.current_frame.push_str(&frame);
        }
    }

    /// Swap to a new effect, advance brand target (OS -> DE -> Kernel -> OS), and restart progress.
    pub fn cycle_effect(&mut self) {
        self.brand_target = self.brand_target.next();
        self.reload_art();
        self.effect = self.playlist.next_effect(&mut self.rng);
        self.dwell_left = self.dwell_time();
        self.cells.settled.fill(0.0);
        self.cells.progress = 0.0;
        self.start_engine_session();
    }

    /// Advance brand target (OS -> DE -> Kernel -> OS) while keeping a pinned effect.
    pub fn cycle_target(&mut self) {
        self.brand_target = self.brand_target.next();
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
        self.start_dwell = false;
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
        self.start_dwell = false;
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
