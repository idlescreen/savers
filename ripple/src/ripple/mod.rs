//! Rain on a still pond — continuous rings, interference, weather states.
//! OS logo sits in the water and warps when rings pass through it.

mod draw;
mod physics;
mod types;
mod update;

pub use types::{DelayedRing, Drop, Ring, Splash, Weather};

use crate::runner::Screensaver;
use crate::runner::{LcgRng, TerminalCell};
use crate::runner::{get_system_info, query_current_palette};
use std::time::Duration;

pub struct Ripple {
    rng: LcgRng,
    time: f32,
    intro_fade: f32,
    last_cols: usize,
    last_rows: usize,
    rings: Vec<Ring>,
    drops: Vec<Drop>,
    splashes: Vec<Splash>,
    delayed: Vec<DelayedRing>,
    rain_timer: f32,
    wind: f32,
    weather: Weather,
    weather_timer: f32,
    weather_intensity: f32,
    on_battery: bool,
    quality_scale: f32,
    frame_time_ema: f32,
    target_frame_time: f32,
    sys_timer: f32,
    accent: (u8, u8, u8),
    logo_text: String,
    surface_phase: f32,
    // Ring-energy accumulation scratch — reused across frames (draw is
    // &self, so interior mutability). Sized cols*rows on demand.
    accum: std::cell::RefCell<Vec<f32>>,
    accent_a: std::cell::RefCell<Vec<f32>>,
}

impl Default for Ripple {
    fn default() -> Self {
        Self::new()
    }
}

impl Ripple {
    pub fn new() -> Self {
        let sys = get_system_info();
        Self {
            rng: LcgRng::from_env_or_random(),
            time: 0.0,
            intro_fade: 0.0,
            last_cols: 0,
            last_rows: 0,
            rings: Vec::new(),
            drops: Vec::new(),
            splashes: Vec::new(),
            delayed: Vec::new(),
            rain_timer: 0.1,
            wind: 0.0,
            weather: Weather::Drizzle,
            weather_timer: 12.0,
            weather_intensity: 0.45,
            on_battery: sys.power_status.contains("Battery"),
            quality_scale: 1.0,
            frame_time_ema: 0.016,
            target_frame_time: 0.016,
            sys_timer: 0.0,
            accent: query_current_palette().accent,
            logo_text: sys.logo_text,
            surface_phase: 0.0,
            accum: std::cell::RefCell::new(Vec::new()),
            accent_a: std::cell::RefCell::new(Vec::new()),
        }
    }

    /// Pin the saver into a stable state for bench harness measurements.
    /// See `cosmos::Cosmos::prepare_for_bench` for the rationale; the
    /// important bit is `sys_refresh_timer = -1000.0` to suppress the
    /// slow system-info probe inside `update()`.
    pub fn prepare_for_bench(&mut self, cols: usize, rows: usize) {
        self.sys_timer = -1000.0;
        self.last_cols = cols;
        self.last_rows = rows;
    }

    /// Inject a ripple at a grid coordinate. Public API for plugin callers
    /// (and the unit tests); `cargo check --all-targets` counts tests as uses
    /// so this is reachable, but `cargo check` alone flags it as a warning.
    #[allow(dead_code)]
    pub fn impact(&mut self, x: f32, y: f32, strength: f32, cols: usize, rows: usize) {
        types::impact(
            &mut self.rings,
            &mut self.splashes,
            &mut self.delayed,
            &mut self.rng,
            self.weather,
            self.on_battery,
            self.quality_scale,
            x,
            y,
            strength,
            cols,
            rows,
        );
    }

    fn pick_weather(&mut self) {
        let r = self.rng.next_f32();
        self.weather = if r < 0.22 {
            Weather::Lull
        } else if r < 0.62 {
            Weather::Drizzle
        } else {
            Weather::Shower
        };
        self.weather_timer = match self.weather {
            Weather::Lull => 6.0 + self.rng.next_f32() * 10.0,
            Weather::Drizzle => 10.0 + self.rng.next_f32() * 16.0,
            Weather::Shower => 5.0 + self.rng.next_f32() * 9.0,
        };
        self.weather_intensity = match self.weather {
            Weather::Lull => 0.15,
            Weather::Drizzle => 0.45,
            Weather::Shower => 0.9,
        };
    }
}

impl Screensaver for Ripple {
    fn init(&mut self, cols: usize, rows: usize) {
        self.intro_fade = 0.0;
        self.time = 0.0;
        self.last_cols = cols;
        self.last_rows = rows;
        self.rings.clear();
        self.drops.clear();
        self.splashes.clear();
        self.delayed.clear();
        self.rain_timer = 0.05;
        self.wind = (self.rng.next_f32() - 0.5) * 0.6;
        self.surface_phase = self.rng.next_f32() * std::f32::consts::TAU;
        self.pick_weather();
        for _ in 0..3 {
            let x = self.rng.next_f32() * cols as f32;
            let y = self.rng.next_f32() * rows as f32;
            let strength = 0.5 + self.rng.next_f32() * 0.4;
            types::spawn_ring(
                &mut self.rings,
                self.weather,
                self.on_battery,
                self.quality_scale,
                x,
                y,
                strength,
                cols,
                rows,
            );
            if let Some(r) = self.rings.last_mut() {
                r.r = self.rng.next_f32() * r.max_r * 0.5;
                r.life = 0.4 + self.rng.next_f32() * 0.5;
                r.age = 0.5;
            }
        }
    }

    fn update_frame_time(&mut self, dt: Duration) {
        let dt_secs = dt.as_secs_f32();
        self.frame_time_ema = self.frame_time_ema * 0.9 + dt_secs.min(0.2) * 0.1;
        if self.time > 1.5 {
            let d = dt_secs * if self.on_battery { 0.65 } else { 1.0 };
            if self.frame_time_ema > self.target_frame_time * 1.15 {
                self.quality_scale = (self.quality_scale - 0.12 * d).max(0.3);
            } else if self.frame_time_ema < self.target_frame_time * 1.05 {
                self.quality_scale = (self.quality_scale + 0.04 * d).min(1.0);
            }
        }
    }

    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let dt_secs = dt.as_secs_f32().min(0.1);
        let speed = if self.on_battery { 0.8 } else { 1.0 };
        let delta = dt_secs * speed;
        self.time += delta;
        self.surface_phase += delta * 0.7;
        if self.intro_fade < 1.0 {
            self.intro_fade = (self.intro_fade + delta / 0.45).min(1.0);
        }
        if cols != self.last_cols || rows != self.last_rows {
            self.init(cols, rows);
            return;
        }

        self.sys_timer += delta;
        if self.sys_timer >= 1.5 {
            self.sys_timer = 0.0;
            let sys = get_system_info();
            self.on_battery = sys.power_status.contains("Battery");
            self.accent = query_current_palette().accent;
            self.logo_text = sys.logo_text;
        }

        self.weather_timer -= delta;
        if self.weather_timer <= 0.0 {
            self.pick_weather();
        }

        let on_battery = self.on_battery;
        let quality_scale = self.quality_scale;
        let state = update::RippleState {
            rings: &mut self.rings,
            drops: &mut self.drops,
            splashes: &mut self.splashes,
            delayed: &mut self.delayed,
            wind: &mut self.wind,
            weather: &mut self.weather,
            weather_intensity: &mut self.weather_intensity,
            rain_timer: &mut self.rain_timer,
        };

        update::update_simulation(
            state,
            &mut self.rng,
            cols,
            rows,
            quality_scale,
            on_battery,
            delta,
        );
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        draw::draw_impl(
            grid,
            cols,
            rows,
            &self.rings,
            &self.drops,
            &self.splashes,
            self.intro_fade,
            self.surface_phase,
            self.weather_intensity,
            self.wind,
            self.accent,
            &self.logo_text,
            &self.accum,
            &self.accent_a,
        );
    }
}

#[cfg(test)]
#[path = "ripple_tests.rs"]
mod tests;
