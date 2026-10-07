// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Effect identity and dispatch.
//!
//! Effects are free functions selected by [`EffectKind`] rather than trait
//! objects, so switching effects in `random` mode never allocates.

use super::cell_state::CellState;
use super::effects;
use crate::runner::LcgRng;

/// One animated treatment of the art.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectKind {
    Decrypt,
    Matrix,
    Scramble,
    Wave,
    Led,
    Shower,
}

impl EffectKind {
    pub const ALL: [EffectKind; 6] = [
        EffectKind::Decrypt,
        EffectKind::Matrix,
        EffectKind::Scramble,
        EffectKind::Wave,
        EffectKind::Led,
        EffectKind::Shower,
    ];

    /// Parse a `[saver] ascii.effect` value.
    pub fn parse(name: &str) -> Option<Self> {
        let key = name.trim().to_ascii_lowercase();
        Self::ALL.into_iter().find(|k| k.name() == key)
    }

    pub fn name(&self) -> &'static str {
        match self {
            EffectKind::Decrypt => "decrypt",
            EffectKind::Matrix => "matrix",
            EffectKind::Scramble => "scramble",
            EffectKind::Wave => "wave",
            EffectKind::Led => "led",
            EffectKind::Shower => "shower",
        }
    }

    /// Seconds this effect holds before `random` mode cuts to the next.
    pub fn dwell(&self) -> f32 {
        match self {
            EffectKind::Decrypt => 6.0,
            EffectKind::Matrix => 9.0,
            EffectKind::Scramble => 11.0,
            EffectKind::Wave => 12.0,
            EffectKind::Led => 8.0,
            EffectKind::Shower => 10.0,
        }
    }
}

/// Draw an effect at random from [`EffectKind::ALL`].
pub fn pick_random(rng: &mut LcgRng) -> EffectKind {
    EffectKind::ALL[rng.next_usize(EffectKind::ALL.len())]
}

/// Advance `st` by one frame under `kind`.
///
/// All randomness flows through `rng` so headless render stays deterministic
/// for a given seed.
pub fn advance(
    kind: EffectKind,
    st: &mut CellState,
    dt: f32,
    t: f32,
    ramp: &[char],
    rng: &mut LcgRng,
) {
    if ramp.is_empty() || st.current.is_empty() {
        return;
    }
    match kind {
        EffectKind::Decrypt => effects::decrypt::advance(st, dt, t, ramp, rng),
        EffectKind::Matrix => effects::matrix::advance(st, dt, t, ramp, rng),
        EffectKind::Scramble => effects::scramble::advance(st, dt, t, ramp, rng),
        EffectKind::Wave => effects::wave::advance(st, dt, t, ramp, rng),
        EffectKind::Led => effects::led::advance(st, dt, t, ramp, rng),
        EffectKind::Shower => effects::shower::advance(st, dt, t, ramp, rng),
    }
}

#[cfg(test)]
mod effect_tests {
    use super::EffectKind;

    #[test]
    fn every_effect_round_trips_through_its_name() {
        for kind in EffectKind::ALL {
            assert_eq!(EffectKind::parse(kind.name()), Some(kind));
        }
    }

    #[test]
    fn unknown_name_is_rejected() {
        assert_eq!(EffectKind::parse("nope"), None);
    }

    #[test]
    fn every_effect_has_a_positive_dwell() {
        for kind in EffectKind::ALL {
            assert!(kind.dwell() > 0.0);
        }
    }
}