// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use super::super::Ascii;
use super::super::cell_state::CellState;
use super::super::effect::{self, EffectKind};
use super::super::load_art;
use super::super::ramp::Ramps;
use crate::runner::{LcgRng, Screensaver, TerminalCell};
use std::time::Duration;

const COLS: usize = 80;
const ROWS: usize = 24;

fn art() -> Vec<String> {
    vec!["IDLE".to_string(), "SAVER".to_string()]
}

fn step(saver: &mut Ascii, frames: usize) {
    let mut grid = vec![TerminalCell::default(); COLS * ROWS];
    for _ in 0..frames {
        saver.update(Duration::from_millis(16), COLS, ROWS);
        saver.draw(&mut grid, COLS, ROWS);
    }
}

#[test]
fn every_effect_survives_a_full_dwell_without_panicking() {
    let ramp = Ramps::load().pick("blocks").to_vec();
    for kind in EffectKind::ALL {
        let mut st = CellState::new();
        st.resize(COLS, ROWS);
        st.load(&art());
        let mut rng = LcgRng::new(7);
        let mut t = 0.0f32;
        for _ in 0..600 {
            effect::advance(kind, &mut st, 0.016, t, &ramp, &mut rng);
            t += 0.016;
        }
    }
}

#[test]
fn decrypt_converges_on_the_target_art() {
    let ramp = Ramps::load().pick("blocks").to_vec();
    let mut st = CellState::new();
    st.resize(COLS, ROWS);
    st.load(&art());
    let mut rng = LcgRng::new(11);
    let mut t = 0.0f32;
    for _ in 0..900 {
        effect::advance(EffectKind::Decrypt, &mut st, 0.016, t, &ramp, &mut rng);
        t += 0.016;
    }
    let mismatches: Vec<(usize, char, char, f32)> = st
        .current
        .iter()
        .zip(st.target.iter())
        .enumerate()
        .filter(|(_, (c, t))| c != t)
        .map(|(i, (c, t))| (i, *c, *t, st.settled[i]))
        .collect();
    assert!(
        mismatches.is_empty(),
        "decrypt should settle exactly on the art; stragglers (idx, drawn, want, settled): {mismatches:?}"
    );
}

#[test]
fn empty_art_leaves_the_grid_blank() {
    let ramp = Ramps::load().pick("blocks").to_vec();
    let mut st = CellState::new();
    st.resize(COLS, ROWS);
    st.load(&[]);
    let mut rng = LcgRng::new(3);
    for kind in EffectKind::ALL {
        effect::advance(kind, &mut st, 0.016, 1.0, &ramp, &mut rng);
    }
    assert!(st.current.iter().all(|c| *c == ' '));
}

#[test]
fn render_is_deterministic_for_a_fixed_seed() {
    let render = |seed: u64| {
        let mut saver = Ascii::new();
        saver.rng = LcgRng::new(seed);
        saver.init(COLS, ROWS);
        // Fix the opening effect, then re-enable rotation so the seeded RNG
        // drives every later pick. Without this the starting effect is
        // seed-independent and the comparison is meaningless.
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
    // The sequence must actually contain art, or this asserts nothing.
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
    // Regression: the default text once came from the session logo, which on
    // a Linux host is the OS pretty name. That block-renders ~195 columns
    // wide, so `CellState::load` rejected it and the saver drew a blank screen
    // on every grid narrower than that — i.e. most real terminals.
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
    // Same failure mode reached through `[saver] ascii.text`.
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
fn zero_sized_grid_does_not_panic() {
    let mut saver = Ascii::new();
    saver.init(0, 0);
    let mut grid: Vec<TerminalCell> = Vec::new();
    saver.update(Duration::from_millis(16), 0, 0);
    saver.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}
