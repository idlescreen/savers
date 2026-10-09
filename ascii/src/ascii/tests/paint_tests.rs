// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use crate::ascii::cell_state::CellState;
use crate::ascii::draw::{paint, paint_frame, paint_transition, xterm_to_rgb};
use crate::runner::TerminalCell;

#[test]
fn paints_art_and_blanks_the_tail() {
    let mut st = CellState::new();
    st.resize(3, 1);
    st.load(&["ABC".to_string()]);
    let mut grid = vec![TerminalCell::default(); 5];
    paint(&st, &mut grid, (1, 2, 3), (0, 0, 0));

    assert_eq!(grid[0].ch, 'A');
    assert_eq!(grid[0].fg, (1, 2, 3));
    assert_eq!(grid[2].ch, 'C');
    assert_eq!(grid[3].ch, ' ');
}

#[test]
fn paint_frame_parses_ansi_colors_and_bold() {
    let frame = "\x1b[1m\x1b[38;2;255;100;50mX\x1b[0m Y\nZ";
    let mut grid = vec![TerminalCell::default(); 6];
    paint_frame(frame, &mut grid, 3, 2, (10, 10, 10), (0, 0, 0));

    assert_eq!(grid[0].ch, 'X');
    assert_eq!(grid[0].fg, (255, 100, 50));
    assert!(grid[0].bold);

    assert_eq!(grid[1].ch, ' ');
    assert_eq!(grid[2].ch, 'Y');
    assert!(!grid[2].bold);

    assert_eq!(grid[3].ch, 'Z');
}

#[test]
fn paint_frame_parses_standard_ansi_colors_and_swallows_csi() {
    let frame = "\x1b[2J\x1b[31mA\x1b[92mB\x1b[0m";
    let mut grid = vec![TerminalCell::default(); 2];
    paint_frame(frame, &mut grid, 2, 1, (10, 10, 10), (0, 0, 0));
    assert_eq!(grid[0].ch, 'A');
    assert_eq!(grid[0].fg, xterm_to_rgb(1));
    assert_eq!(grid[1].ch, 'B');
    assert_eq!(grid[1].fg, xterm_to_rgb(10));
}

#[test]
fn paint_frame_preserves_ansi_state_across_newlines() {
    let frame = "\x1b[31mA\nB\x1b[0m";
    let mut grid = vec![TerminalCell::default(); 4];
    paint_frame(frame, &mut grid, 2, 2, (10, 10, 10), (0, 0, 0));
    assert_eq!(grid[0].fg, xterm_to_rgb(1));
    assert_eq!(grid[2].fg, xterm_to_rgb(1));
}

#[test]
fn paint_transition_blends_smoothly_without_panics() {
    let from = vec![
        TerminalCell {
            ch: 'F',
            fg: (200, 100, 50),
            bg: (0, 0, 0),
            bold: false,
        };
        4
    ];
    let to = vec![
        TerminalCell {
            ch: 'G',
            fg: (50, 100, 200),
            bg: (0, 0, 0),
            bold: false,
        };
        4
    ];
    let mut out = vec![TerminalCell::default(); 4];

    // At t=0.0: should reflect from
    paint_transition(&from, &to, &mut out, 0.0, 2, 2);
    assert_eq!(out[0].ch, 'F');
    assert_eq!(out[0].fg, (200, 100, 50));

    // At t=1.0: should reflect to
    paint_transition(&from, &to, &mut out, 1.0, 2, 2);
    assert_eq!(out[0].ch, 'G');
    assert_eq!(out[0].fg, (50, 100, 200));

    // At midpoint: no panic, valid characters
    paint_transition(&from, &to, &mut out, 0.5, 2, 2);
    assert!(out.iter().all(|c| c.ch == 'F' || c.ch == 'G'));
}
