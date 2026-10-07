// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Paints [`CellState`] into the caller's terminal grid.
//!
//! Writes only — never allocates, never resizes `grid`. Cells the state does
//! not cover are reset to the default blank so a resize can never leave stale
//! glyphs behind.

use super::cell_state::CellState;
use crate::runner::TerminalCell;

pub fn paint(
    st: &CellState,
    grid: &mut [TerminalCell],
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
) {
    let n = st.current.len().min(grid.len());
    for (cell, ch) in grid.iter_mut().zip(st.current.iter()) {
        cell.ch = *ch;
        cell.fg = fg;
        cell.bg = bg;
        cell.bold = false;
    }
    for cell in grid.iter_mut().skip(n) {
        *cell = TerminalCell::default();
    }
}

#[cfg(test)]
mod paint_tests {
    use super::paint;
    use crate::ascii::cell_state::CellState;
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
        assert_eq!(grid[4].ch, ' ');
    }
}