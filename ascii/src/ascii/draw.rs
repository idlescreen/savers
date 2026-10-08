// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Paints formatted frames and [`CellState`] into the terminal grid.

use super::cell_state::CellState;
use crate::runner::TerminalCell;

/// Paints a raw CellState (used for fallback or tests).
pub fn paint(st: &CellState, grid: &mut [TerminalCell], fg: (u8, u8, u8), bg: (u8, u8, u8)) {
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

/// Parses an ANSI-formatted ttfx frame string directly into the grid.
///
/// Steady-state zero heap allocation: parses bytes in-place.
pub fn paint_frame(
    frame: &str,
    grid: &mut [TerminalCell],
    cols: usize,
    rows: usize,
    default_fg: (u8, u8, u8),
    default_bg: (u8, u8, u8),
) {
    grid.fill(TerminalCell {
        ch: ' ',
        fg: default_fg,
        bg: default_bg,
        bold: false,
    });
    if cols == 0 || rows == 0 {
        return;
    }
    let mut x = 0usize;
    let mut y = 0usize;
    let mut bold = false;
    let mut fg = default_fg;
    let mut bg = default_bg;

    let bytes = frame.as_bytes();
    let mut i = 0;
    while i < bytes.len() && y < rows {
        if bytes[i] == b'\n' {
            y += 1;
            x = 0;
            i += 1;
            continue;
        }
        if bytes[i] == 0x1b {
            i += 1;
            if i < bytes.len() && bytes[i] == b'[' {
                i += 1;
                let start = i;
                while i < bytes.len() && (bytes[i] < b'@' || bytes[i] > b'~') {
                    i += 1;
                }
                if i < bytes.len() {
                    if bytes[i] == b'm' {
                        let seq = &bytes[start..i];
                        parse_sgr(seq, &mut fg, &mut bg, &mut bold, default_fg, default_bg);
                    }
                    i += 1;
                }
            } else if i < bytes.len() {
                i += 1;
            }
            continue;
        }
        let lead = bytes[i];
        let len = if lead < 0x80 {
            1
        } else if lead < 0xe0 {
            2
        } else if lead < 0xf0 {
            3
        } else {
            4
        };
        let end = (i + len).min(bytes.len());
        if let Ok(s) = std::str::from_utf8(&bytes[i..end]) {
            if let Some(ch) = s.chars().next() {
                if x < cols && y < rows {
                    let idx = y * cols + x;
                    if idx < grid.len() {
                        grid[idx] = TerminalCell { ch, fg, bg, bold };
                    }
                }
                x += 1;
            }
        }
        i = end;
    }
}

fn parse_sgr(
    seq: &[u8],
    fg: &mut (u8, u8, u8),
    bg: &mut (u8, u8, u8),
    bold: &mut bool,
    default_fg: (u8, u8, u8),
    default_bg: (u8, u8, u8),
) {
    if seq.is_empty() || seq == b"0" {
        *fg = default_fg;
        *bg = default_bg;
        *bold = false;
        return;
    }
    let mut parts = seq.split(|&b| b == b';');
    while let Some(part) = parts.next() {
        match part {
            b"0" => {
                *fg = default_fg;
                *bg = default_bg;
                *bold = false;
            }
            b"1" => *bold = true,
            b"22" => *bold = false,
            [b'3', d @ b'0'..=b'7'] => *fg = xterm_to_rgb(d - b'0'),
            [b'9', d @ b'0'..=b'7'] => *fg = xterm_to_rgb(8 + (d - b'0')),
            [b'4', d @ b'0'..=b'7'] => *bg = xterm_to_rgb(d - b'0'),
            [b'1', b'0', d @ b'0'..=b'7'] => *bg = xterm_to_rgb(8 + (d - b'0')),
            b"38" => match parts.next() {
                Some(b"2") => {
                    let r = parts.next().and_then(parse_u8).unwrap_or(default_fg.0);
                    let g = parts.next().and_then(parse_u8).unwrap_or(default_fg.1);
                    let b = parts.next().and_then(parse_u8).unwrap_or(default_fg.2);
                    *fg = (r, g, b);
                }
                Some(b"5") => {
                    if let Some(code) = parts.next().and_then(parse_u8) {
                        *fg = xterm_to_rgb(code);
                    }
                }
                _ => {}
            },
            b"48" => match parts.next() {
                Some(b"2") => {
                    let r = parts.next().and_then(parse_u8).unwrap_or(default_bg.0);
                    let g = parts.next().and_then(parse_u8).unwrap_or(default_bg.1);
                    let b = parts.next().and_then(parse_u8).unwrap_or(default_bg.2);
                    *bg = (r, g, b);
                }
                Some(b"5") => {
                    if let Some(code) = parts.next().and_then(parse_u8) {
                        *bg = xterm_to_rgb(code);
                    }
                }
                _ => {}
            },
            b"39" => *fg = default_fg,
            b"49" => *bg = default_bg,
            _ => {}
        }
    }
}

fn parse_u8(bytes: &[u8]) -> Option<u8> {
    if bytes.is_empty() || bytes.len() > 3 {
        return None;
    }
    let mut val = 0u16;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        val = val * 10 + (b - b'0') as u16;
    }
    if val <= 255 {
        Some(val as u8)
    } else {
        None
    }
}

fn xterm_to_rgb(code: u8) -> (u8, u8, u8) {
    let hex = ttfx::utils::hexterm::xterm_to_hex(code);
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else {
        (255, 255, 255)
    }
}

#[cfg(test)]
mod paint_tests {
    use super::*;
    use crate::ascii::cell_state::CellState;

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
}
