// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use super::super::Ascii;
use super::super::cell_state::CellState;
use super::super::effect::EffectKind;
use super::super::load_art;
use crate::runner::{LcgRng, Screensaver, TerminalCell};
use clap::Parser;
use std::time::Duration;

const COLS: usize = 80;
const ROWS: usize = 24;

fn step(saver: &mut Ascii, frames: usize) {
    let mut grid = vec![TerminalCell::default(); COLS * ROWS];
    for _ in 0..frames {
        saver.update(Duration::from_millis(16), COLS, ROWS);
        saver.draw(&mut grid, COLS, ROWS);
    }
}

#[test]
fn every_effect_survives_running_without_panicking() {
    let mut saver = Ascii::new();
    saver.init(COLS, ROWS);
    let mut grid = vec![TerminalCell::default(); COLS * ROWS];
    for &kind in &EffectKind::ALL {
        saver.pin_effect(kind);
        for _ in 0..10 {
            saver.update(Duration::from_millis(16), COLS, ROWS);
            saver.draw(&mut grid, COLS, ROWS);
        }
    }
}

#[test]
fn empty_art_leaves_the_grid_blank() {
    let mut saver = Ascii::new();
    saver.art_text = String::new();
    saver.init(COLS, ROWS);
    saver.art_text = String::new();
    saver.engine = None;
    let mut grid = vec![TerminalCell::default(); COLS * ROWS];
    saver.draw(&mut grid, COLS, ROWS);
    assert!(grid.iter().all(|c| c.ch == ' '));
}

#[test]
fn render_is_deterministic_for_a_fixed_seed() {
    let render = |seed: u64| {
        let mut saver = Ascii::new();
        saver.rng = LcgRng::new(seed);
        saver.init(COLS, ROWS);
        saver.pin_effect(EffectKind::Decrypt);
        saver.pinned = false;
        let mut grid = vec![TerminalCell::default(); COLS * ROWS];
        for _ in 0..600 {
            saver.update(Duration::from_millis(16), COLS, ROWS);
            saver.draw(&mut grid, COLS, ROWS);
        }
        grid.iter().map(|c| c.ch).collect::<Vec<char>>()
    };
    assert_eq!(render(1234), render(1234));
    assert!(render(1234).contains(&'█'));
}

#[test]
fn resize_rebuilds_the_grid_and_art() {
    let mut saver = Ascii::new();
    saver.init(COLS, ROWS);
    step(&mut saver, 10);

    let (mut wide, mut tall) = (120, 40);
    saver.update(Duration::from_millis(16), wide, tall);
    let mut grid = vec![TerminalCell::default(); wide * tall];
    saver.draw(&mut grid, wide, tall);
    assert_eq!(grid.len(), wide * tall);

    wide = 60;
    tall = 20;
    saver.update(Duration::from_millis(16), wide, tall);
    let mut smaller = vec![TerminalCell::default(); wide * tall];
    saver.draw(&mut smaller, wide, tall);
    assert_eq!(saver.cells.target.len(), wide * tall);
}

#[test]
fn default_art_renders_at_terminal_grid_sizes() {
    for &(cols, rows) in &[(80usize, 24usize), (120, 40), (160, 48)] {
        let lines = load_art::resolve_lines(cols, rows);
        assert!(!lines.is_empty(), "no art resolved at {cols}x{rows}");
        let mut st = CellState::new();
        st.resize(cols, rows);
        st.load(&lines);
        assert!(st.inked_cells() > 0, "blank grid at {cols}x{rows}");
    }
}

#[test]
fn long_configured_text_still_fits_the_grid() {
    let lines = load_art::build_lines("Fedora Linux 44 (Server Edition)", None, 80);
    let mut st = CellState::new();
    st.resize(80, 24);
    st.load(&lines);
    assert!(
        st.inked_cells() > 0,
        "blank grid for overlong configured text"
    );
}

#[test]
fn branding_file_read_and_fallback_to_wordmark() {
    let art = load_art::resolve_art(80, 24);
    assert!(!art.is_empty(), "resolved art must not be empty");
}

#[test]
fn zero_sized_grid_does_not_panic() {
    let mut saver = Ascii::new();
    saver.init(0, 0);
    let mut grid: Vec<TerminalCell> = Vec::new();
    saver.update(Duration::from_millis(16), 0, 0);
    saver.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[rustfmt::skip]
const ALL_37_NAMES: &[&str] = &[
    "beams", "binarypath", "blackhole", "bouncyballs", "bubbles", "burn",
    "colorshift", "crumble", "decrypt", "errorcorrect", "expand", "fireworks",
    "highlight", "laseretch", "matrix", "middleout", "orbittingvolley",
    "overflow", "pour", "print", "rain", "randomsequence", "rings",
    "scattered", "slice", "slide", "smoke", "spotlights", "spray",
    "swarm", "sweep", "synthgrid", "thunderstorm", "unstable", "vhstape",
    "waves", "wipe",
];

#[test]
fn all_37_effects_can_be_parsed() {
    assert_eq!(ALL_37_NAMES.len(), 37);
    for name in ALL_37_NAMES {
        let cli = ttfx::cli::Cli::try_parse_from(["ttfx", name]).unwrap();
        assert!(cli.effect.is_some(), "failed to parse effect: {name}");
        let kind = EffectKind::parse(name);
        assert!(kind.is_some(), "EffectKind::parse failed for {name}");
    }
}

#[test]
fn all_37_effects_can_build_and_render_frames() {
    let art_data = "OMARCHY\nSCREENSAVER";
    for name in ALL_37_NAMES {
        let cli = ttfx::cli::Cli::try_parse_from(["ttfx", name]).unwrap();
        let mut config = cli.terminal_config();
        let cmd = cli.effect.unwrap();
        let mut effect = cmd.build_effect();
        config.canvas_width = 80;
        config.canvas_height = 24;
        config.ignore_terminal_dimensions = true;
        config.anchor_text = ttfx::engine::canvas::Anchor::C;
        let clock = ttfx::engine::ctx::Clock::virtual_with_frame_rate(60);
        let rng = ttfx::utils::rng::Rng::seeded(42);
        let mut ctx = ttfx::engine::ctx::EngineCtx::new(art_data, config, rng, clock).unwrap();
        effect.build(&mut ctx).unwrap();
        for _ in 0..5 {
            let _ = effect.next_frame(&mut ctx);
        }
    }
}

#[test]
fn effect_dwells_on_settled_frame_before_restarting() {
    let mut saver = Ascii::new();
    saver.params.cycle_secs = Some(5.0);
    saver.init(COLS, ROWS);
    saver.pin_effect(EffectKind::Wipe);
    let mut grid = vec![TerminalCell::default(); COLS * ROWS];
    for _ in 0..160 {
        saver.update(Duration::from_millis(16), COLS, ROWS);
        saver.draw(&mut grid, COLS, ROWS);
    }
    assert!(saver.engine.is_none());
    assert!(!saver.current_frame.is_empty());
    assert!(grid.iter().any(|c| c.ch != ' '));
}

#[test]
fn oversized_art_falls_back_without_blanking() {
    let mut saver = Ascii::new();
    saver.init(20, 10);
    assert!(!saver.art_text.trim().is_empty());
    assert!(saver.cells.inked_cells() > 0);
    saver.pin_effect(EffectKind::Decrypt);
    let mut grid = vec![TerminalCell::default(); 20 * 10];
    for _ in 0..30 {
        saver.update(Duration::from_millis(16), 20, 10);
        saver.draw(&mut grid, 20, 10);
    }
    assert!(grid.iter().any(|c| c.ch != ' '));
}
