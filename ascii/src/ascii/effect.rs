// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Effect identity and dispatch for the 37 ttfx visual effects.

use crate::runner::LcgRng;

/// The 37 visual effects supported by ttfx.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectKind {
    Beams,
    Binarypath,
    Blackhole,
    Bouncyballs,
    Bubbles,
    Burn,
    Colorshift,
    Crumble,
    Decrypt,
    Errorcorrect,
    Expand,
    Fireworks,
    Highlight,
    Laseretch,
    Matrix,
    Middleout,
    Orbittingvolley,
    Overflow,
    Pour,
    Print,
    Rain,
    Randomsequence,
    Rings,
    Scattered,
    Slice,
    Slide,
    Smoke,
    Spotlights,
    Spray,
    Swarm,
    Sweep,
    Synthgrid,
    Thunderstorm,
    Unstable,
    Vhstape,
    Waves,
    Wipe,
}

impl EffectKind {
    pub const ALL: [EffectKind; 37] = [
        EffectKind::Beams,
        EffectKind::Binarypath,
        EffectKind::Blackhole,
        EffectKind::Bouncyballs,
        EffectKind::Bubbles,
        EffectKind::Burn,
        EffectKind::Colorshift,
        EffectKind::Crumble,
        EffectKind::Decrypt,
        EffectKind::Errorcorrect,
        EffectKind::Expand,
        EffectKind::Fireworks,
        EffectKind::Highlight,
        EffectKind::Laseretch,
        EffectKind::Matrix,
        EffectKind::Middleout,
        EffectKind::Orbittingvolley,
        EffectKind::Overflow,
        EffectKind::Pour,
        EffectKind::Print,
        EffectKind::Rain,
        EffectKind::Randomsequence,
        EffectKind::Rings,
        EffectKind::Scattered,
        EffectKind::Slice,
        EffectKind::Slide,
        EffectKind::Smoke,
        EffectKind::Spotlights,
        EffectKind::Spray,
        EffectKind::Swarm,
        EffectKind::Sweep,
        EffectKind::Synthgrid,
        EffectKind::Thunderstorm,
        EffectKind::Unstable,
        EffectKind::Vhstape,
        EffectKind::Waves,
        EffectKind::Wipe,
    ];

    /// Parse an effect name or alias into an [`EffectKind`].
    pub fn parse(name: &str) -> Option<Self> {
        let key = name.trim().to_ascii_lowercase();
        let stripped = key.replace(['-', '_'], "");
        match stripped.as_str() {
            "beams" => Some(EffectKind::Beams),
            "binarypath" => Some(EffectKind::Binarypath),
            "blackhole" => Some(EffectKind::Blackhole),
            "bouncyballs" => Some(EffectKind::Bouncyballs),
            "bubbles" => Some(EffectKind::Bubbles),
            "burn" => Some(EffectKind::Burn),
            "colorshift" => Some(EffectKind::Colorshift),
            "crumble" => Some(EffectKind::Crumble),
            "decrypt" => Some(EffectKind::Decrypt),
            "errorcorrect" => Some(EffectKind::Errorcorrect),
            "expand" => Some(EffectKind::Expand),
            "fireworks" => Some(EffectKind::Fireworks),
            "highlight" => Some(EffectKind::Highlight),
            "laseretch" => Some(EffectKind::Laseretch),
            "matrix" => Some(EffectKind::Matrix),
            "middleout" => Some(EffectKind::Middleout),
            "orbittingvolley" => Some(EffectKind::Orbittingvolley),
            "overflow" => Some(EffectKind::Overflow),
            "pour" => Some(EffectKind::Pour),
            "print" => Some(EffectKind::Print),
            "rain" | "shower" => Some(EffectKind::Rain),
            "randomsequence" => Some(EffectKind::Randomsequence),
            "rings" => Some(EffectKind::Rings),
            "scattered" | "scramble" => Some(EffectKind::Scattered),
            "slice" => Some(EffectKind::Slice),
            "slide" => Some(EffectKind::Slide),
            "smoke" => Some(EffectKind::Smoke),
            "spotlights" => Some(EffectKind::Spotlights),
            "spray" => Some(EffectKind::Spray),
            "swarm" => Some(EffectKind::Swarm),
            "sweep" | "led" => Some(EffectKind::Sweep),
            "synthgrid" => Some(EffectKind::Synthgrid),
            "thunderstorm" => Some(EffectKind::Thunderstorm),
            "unstable" => Some(EffectKind::Unstable),
            "vhstape" => Some(EffectKind::Vhstape),
            "waves" | "wave" => Some(EffectKind::Waves),
            "wipe" => Some(EffectKind::Wipe),
            _ => None,
        }
    }

    /// The exact subcommand name accepted by ttfx CLI.
    pub fn name(&self) -> &'static str {
        match self {
            EffectKind::Beams => "beams",
            EffectKind::Binarypath => "binarypath",
            EffectKind::Blackhole => "blackhole",
            EffectKind::Bouncyballs => "bouncyballs",
            EffectKind::Bubbles => "bubbles",
            EffectKind::Burn => "burn",
            EffectKind::Colorshift => "colorshift",
            EffectKind::Crumble => "crumble",
            EffectKind::Decrypt => "decrypt",
            EffectKind::Errorcorrect => "errorcorrect",
            EffectKind::Expand => "expand",
            EffectKind::Fireworks => "fireworks",
            EffectKind::Highlight => "highlight",
            EffectKind::Laseretch => "laseretch",
            EffectKind::Matrix => "matrix",
            EffectKind::Middleout => "middleout",
            EffectKind::Orbittingvolley => "orbittingvolley",
            EffectKind::Overflow => "overflow",
            EffectKind::Pour => "pour",
            EffectKind::Print => "print",
            EffectKind::Rain => "rain",
            EffectKind::Randomsequence => "randomsequence",
            EffectKind::Rings => "rings",
            EffectKind::Scattered => "scattered",
            EffectKind::Slice => "slice",
            EffectKind::Slide => "slide",
            EffectKind::Smoke => "smoke",
            EffectKind::Spotlights => "spotlights",
            EffectKind::Spray => "spray",
            EffectKind::Swarm => "swarm",
            EffectKind::Sweep => "sweep",
            EffectKind::Synthgrid => "synthgrid",
            EffectKind::Thunderstorm => "thunderstorm",
            EffectKind::Unstable => "unstable",
            EffectKind::Vhstape => "vhstape",
            EffectKind::Waves => "waves",
            EffectKind::Wipe => "wipe",
        }
    }

    /// Seconds this effect holds before random mode cycles to the next.
    pub fn dwell(&self) -> f32 {
        match self {
            EffectKind::Decrypt => 8.0,
            EffectKind::Matrix => 10.0,
            EffectKind::Scattered => 10.0,
            EffectKind::Waves => 10.0,
            EffectKind::Sweep => 8.0,
            EffectKind::Rain => 10.0,
            EffectKind::Fireworks => 9.0,
            EffectKind::Thunderstorm => 11.0,
            _ => 10.0,
        }
    }
}

/// Draw an effect at random from [`EffectKind::ALL`].
pub fn pick_random(rng: &mut LcgRng) -> EffectKind {
    EffectKind::ALL[rng.next_usize(EffectKind::ALL.len())]
}
