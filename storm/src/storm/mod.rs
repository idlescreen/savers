//! Consolidated storm screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).

pub mod draw;
pub mod physics;
pub mod types;

use crate::runner::Screensaver;
use crate::runner::{LcgRng, TerminalCell};
use std::time::Duration;

use crate::runner::{get_system_info, query_current_palette};

#[allow(unused_imports)]
pub use self::types::{
    Animal, AnimalState, AnimalType, BirdState, Drop, LogoCell, Phase, SceneryCell, Splash, Star,
};

pub struct Storm {
    pub(crate) rng: LcgRng,
    pub(crate) stars: Vec<Star>,
    pub(crate) logo_cells: Vec<LogoCell>,
    pub(crate) drops: Vec<Drop>,
    pub(crate) splashes: Vec<Splash>,
    pub(crate) phase: Phase,
    pub(crate) phase_timer: f32,
    pub(crate) last_cols: usize,
    pub(crate) last_rows: usize,
    pub(crate) drop_count_opt: u32,
    pub(crate) assemble_speed_opt: u32,

    // Live system dynamics
    pub(crate) sys_refresh_timer: f32,
    pub(crate) mem_pressure: f32,
    pub(crate) cpu_load: f32,
    pub(crate) _host_bias: f32,
    pub(super) on_battery: bool,
    pub(super) frame_time_ema: f32,
    pub(super) quality_scale: f32,
    pub(super) target_frame_time: f32,

    // Puddle accumulation
    pub(crate) puddle: Vec<f32>,
    pub(crate) puddle_color: Vec<(u8, u8, u8)>,

    // Wind dynamics
    pub(crate) wind: f32,

    // Lightning
    pub(crate) lightning_timer: f32,
    pub(crate) lightning_flash: f32,
    pub(crate) lightning_bolts: Vec<Vec<(usize, usize)>>,
    pub(crate) lightning_is_background: bool,
    pub(crate) lightning_delay: f32,

    // Scenery
    pub(crate) bg_cells: Vec<SceneryCell>,
    pub(crate) mid_scenery: Vec<SceneryCell>,
    pub(crate) fg_scenery: Vec<SceneryCell>,

    // Bird state
    pub(crate) bird_x: f32,
    pub(crate) bird_y: f32,
    pub(crate) bird_state: BirdState,
    pub(crate) bird_timer: f32,
    pub(crate) bird_wing_flap: bool,
    pub(crate) bird_vx: f32,
    pub(crate) bird_vy: f32,
    pub(crate) bird_perch_x: f32,
    pub(crate) bird_perch_y: f32,
    pub(crate) perch_points: Vec<(usize, usize)>,

    // Active Animal
    pub(crate) active_animal: Option<Animal>,
    pub(crate) animal_spawn_timer: f32,

    // Subtitles
    pub(crate) subtitle: String,
    pub(crate) subtitle_timer: f32,
    pub(crate) time_elapsed: f32,
    pub(crate) cached_accent: (u8, u8, u8),
    /// 0→1 fade-in after init / resize (~0.45s)
    pub(crate) intro_fade: f32,
    /// Smoothed wind target (actual wind eases toward this)
    pub(crate) wind_target: f32,
}

impl Default for Storm {
    fn default() -> Self {
        Self::new()
    }
}

impl Storm {
    pub fn new() -> Self {
        // Pre-4.1 HKEY_CURRENT_USER registry reads (DropCount, AssembleSpeed)
        // collapsed to defaults for the inline migration. Re-added in 4.2.
        let drop_count_opt: u32 = 1;
        let assemble_speed_opt: u32 = 1;

        let sys = get_system_info();
        let on_battery = sys.power_status.contains("Battery");

        Self {
            rng: LcgRng::from_env_or_random(),
            stars: Vec::new(),
            logo_cells: Vec::new(),
            drops: Vec::new(),
            splashes: Vec::new(),
            phase: Phase::Building,
            phase_timer: 0.0,
            last_cols: 0,
            last_rows: 0,
            drop_count_opt,
            assemble_speed_opt,
            sys_refresh_timer: 0.0,
            mem_pressure: sys.mem_used_pct / 100.0,
            cpu_load: (sys.cpu_usage_pct / 100.0).clamp(0.0, 1.0),
            _host_bias: sys.hostname.chars().map(|c| c as u32).sum::<u32>() as f32 / 1000.0 % 1.0,
            on_battery,
            frame_time_ema: 0.01666667,
            quality_scale: 1.0,
            target_frame_time: 0.01666667,
            puddle: Vec::new(),
            puddle_color: Vec::new(),
            wind: 0.0,
            lightning_timer: 0.0,
            lightning_flash: 0.0,
            lightning_bolts: Vec::new(),
            lightning_is_background: false,
            lightning_delay: 0.0,
            bg_cells: Vec::new(),
            mid_scenery: Vec::new(),
            fg_scenery: Vec::new(),
            bird_x: 0.0,
            bird_y: 0.0,
            bird_state: BirdState::Sitting,
            bird_timer: 0.0,
            bird_wing_flap: false,
            bird_vx: 0.0,
            bird_vy: 0.0,
            bird_perch_x: 0.0,
            bird_perch_y: 0.0,
            perch_points: Vec::new(),
            active_animal: None,
            animal_spawn_timer: 28.0,
            subtitle: String::new(),
            subtitle_timer: 0.0,
            time_elapsed: 0.0,
            cached_accent: query_current_palette().accent,
            intro_fade: 0.0,
            wind_target: 0.0,
        }
    }

    /// Pin the saver into a stable state for bench harness measurements.
    /// See `cosmos::Cosmos::prepare_for_bench` for the rationale; the
    /// important bit is `sys_refresh_timer = -1000.0` to suppress the
    /// slow system-info probe inside `update()`.
    pub fn prepare_for_bench(&mut self, cols: usize, rows: usize) {
        self.sys_refresh_timer = -1000.0;
        self.last_cols = cols;
        self.last_rows = rows;
    }
}

impl Screensaver for Storm {
    fn update_frame_time(&mut self, dt: Duration) {
        let dt_secs = dt.as_secs_f32();

        // Auto-detect high refresh rates during the startup phase
        if self.phase_timer < 2.0 && dt_secs > 0.001 && dt_secs < self.target_frame_time - 0.001 {
            self.target_frame_time = dt_secs;
        }

        // Exponential moving average for frame time (alpha = 0.1)
        self.frame_time_ema = self.frame_time_ema * 0.9 + dt_secs.min(0.2) * 0.1;

        if self.phase_timer > 1.5 {
            let speed_mult = if self.on_battery { 0.65 } else { 1.0 };
            let delta = dt_secs * speed_mult;
            if self.frame_time_ema > self.target_frame_time * 1.15 {
                self.quality_scale = (self.quality_scale - 0.15 * delta).max(0.20);
            } else if self.frame_time_ema < self.target_frame_time * 1.05 {
                self.quality_scale = (self.quality_scale + 0.04 * delta).min(1.0);
            }
        }
    }

    fn init(&mut self, _cols: usize, _rows: usize) {
        // Leave last_cols/last_rows so the next update's check_resize rebuilds scenery.
        self.intro_fade = 0.0;
        self.time_elapsed = 0.0;
        self.phase_timer = 0.0;
        self.drops.clear();
        self.splashes.clear();
        self.lightning_flash = 0.0;
        self.lightning_bolts.clear();
        self.last_cols = 0;
        self.last_rows = 0;
    }

    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let dt_secs = dt.as_secs_f32().min(0.1);
        let speed_mult = if self.on_battery { 0.65 } else { 1.0 };
        let delta = dt_secs * speed_mult;
        self.phase_timer += delta;
        self.time_elapsed += delta;

        // Intro fade ~0.45s
        if self.intro_fade < 1.0 {
            self.intro_fade = (self.intro_fade + delta / 0.45).min(1.0);
        }

        // Wind target drifts with slow LFO; actual wind eases toward it (no snaps)
        self.wind_target =
            (self.phase_timer * 0.35).sin() * 9.0 + (self.phase_timer * 1.5).cos() * 2.0;
        let wind_ease = 1.0 - (-delta * 1.4).exp();
        self.wind += (self.wind_target - self.wind) * wind_ease;

        self.sys_refresh_timer += delta;
        if self.sys_refresh_timer >= 1.0 {
            let sys = get_system_info();
            self.mem_pressure = sys.mem_used_pct / 100.0;
            self.cpu_load = (sys.cpu_usage_pct / 100.0).clamp(0.0, 1.0);
            self.on_battery = sys.power_status.contains("Battery");
            self.cached_accent = query_current_palette().accent;
            self.sys_refresh_timer = 0.0;
        }

        self.check_resize(cols, rows);

        let load_mult = 1.0 + self.cpu_load * 0.6 + self.mem_pressure * 0.3;
        let speed_mult = match self.assemble_speed_opt {
            0 => 0.6f32,
            2 => 1.6f32,
            _ => 1.0f32,
        } * load_mult;

        self.update_drops(delta, cols, rows, speed_mult);
        if !crate::runner::is_secondary_monitor() {
            self.update_bird(delta, cols, rows);
            self.update_scenery_and_animals(delta, cols, rows);
        }
        self.update_lightning(delta, cols, rows);
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        self.draw_impl(grid, cols, rows);
    }
}

#[cfg(test)]
#[path = "storm_tests.rs"]
mod tests;
