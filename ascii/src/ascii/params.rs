// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! `[saver] ascii.*` parameter parsing.
//!
//! Read once at construction. The daemon surfaces `config.yaml` values through
//! the environment, so this never touches the filesystem.

use super::effect::EffectKind;
use crate::runner::{param, param_f32};

const SPEED_DEFAULT: f32 = 1.0;
const SPEED_MIN: f32 = 0.15;
const SPEED_MAX: f32 = 4.0;

pub struct Params {
    /// `None` means `random`: cycle through every effect.
    pub effect: Option<EffectKind>,
    pub ramp: String,
    pub speed: f32,
    /// Explicit `[saver] ascii.fg`; `None` follows the theme accent.
    pub fg: Option<(u8, u8, u8)>,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            effect: None,
            ramp: super::ramp::Ramps::default_name().to_string(),
            speed: SPEED_DEFAULT,
            fg: None,
        }
    }
}

impl Params {
    pub fn read() -> Self {
        // `random` and an unset value both mean "cycle every effect".
        let effect = match param("effect").as_deref().map(str::trim) {
            Some("random") | None => None,
            Some(name) => EffectKind::parse(name),
        };

        let ramp = param("ramp")
            .map(|r| r.trim().to_string())
            .filter(|r| !r.is_empty())
            .unwrap_or_else(|| super::ramp::Ramps::default_name().to_string());

        let speed = param_f32("speed")
            .unwrap_or(SPEED_DEFAULT)
            .clamp(SPEED_MIN, SPEED_MAX);

        let fg = param("fg").as_deref().and_then(parse_hex);

        Self {
            effect,
            ramp,
            speed,
            fg,
        }
    }
}

/// Accepts `#rrggbb` or bare `rrggbb`.
pub fn parse_hex(raw: &str) -> Option<(u8, u8, u8)> {
    let hex = raw.trim().strip_prefix('#').unwrap_or(raw.trim());
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

#[cfg(test)]
mod params_tests {
    use super::parse_hex;

    #[test]
    fn parses_six_digit_hex_with_and_without_hash() {
        assert_eq!(parse_hex("#ff8000"), Some((255, 128, 0)));
        assert_eq!(parse_hex("FF8000"), Some((255, 128, 0)));
    }

    #[test]
    fn rejects_malformed_hex_without_panicking() {
        assert_eq!(parse_hex("#fff"), None);
        assert_eq!(parse_hex("gggggg"), None);
        assert_eq!(parse_hex(""), None);
    }
}