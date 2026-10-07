// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Resolves the art the saver animates.
//!
//! Input is a `[saver]` param, never a file: reading one would require a
//! `filesystem_read` capability grant, and every other saver ships with
//! `filesystem_read = []`. Keeping the plugin at zero filesystem access is
//! deliberate — see the plan's asset-delivery decision.

use crate::runner::{param, render_logo_block};

/// `[saver] ascii.text`, falling back to the shared wordmark.
///
/// This used to hardcode its own string while the other eleven savers read
/// `SystemInfo::logo_text`, so one machine showed different words depending on
/// which saver you happened to land on. There is one wordmark now; every saver
/// resolves it the same way.
pub fn resolve_text() -> String {
    param("text")
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(crate::runner::wordmark)
}

/// Optional second line beneath the art block.
pub fn resolve_sub_text() -> Option<String> {
    param("sub")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Render the resolved text or custom art, sized to fit a `cols`×`rows` grid.
pub fn resolve_lines(cols: usize, rows: usize) -> Vec<String> {
    let sub = resolve_sub_text();
    if param("text").is_none() {
        if let Some(art) = crate::runner::logo() {
            if let Some(lines) = parse_art(art, sub.as_deref(), cols, rows) {
                return lines;
            }
        }
    }
    build_lines(&resolve_text(), sub.as_deref(), cols)
}

/// Pure custom art parser, validating dimensions against the grid.
pub fn parse_art(art: &str, sub: Option<&str>, cols: usize, rows: usize) -> Option<Vec<String>> {
    let mut lines: Vec<String> = art.lines().map(|l| l.trim_end().to_string()).collect();
    while lines.first().is_some_and(|l| l.is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    if lines.is_empty() {
        return None;
    }
    if let Some(s) = sub {
        lines.push(String::new());
        lines.push(s.to_string());
    }
    let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let height = lines.len();
    if width > 0 && width <= cols && height <= rows {
        Some(lines)
    } else {
        None
    }
}

/// Pure art builder, split out so it is testable without touching the
/// environment `param()` reads.
///
/// Long text is trimmed a character at a time until the rendered block fits
/// `max_cols`. Rendering happens once at init, so the retry loop is cheap and
/// it guarantees `CellState::load` never silently drops oversized art and
/// leaves the screen blank.
pub fn build_lines(text: &str, sub: Option<&str>, max_cols: usize) -> Vec<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let mut chars: Vec<char> = trimmed.chars().collect();
    loop {
        let candidate: String = chars.iter().collect();
        let lines = render_logo_block(&candidate, sub);
        if lines.iter().all(|l| l.trim().is_empty()) {
            return Vec::new();
        }
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        if width <= max_cols || chars.is_empty() {
            return lines;
        }
        chars.pop();
    }
}

#[cfg(test)]
mod art_tests {
    use super::build_lines;

    #[test]
    fn blank_text_renders_no_lines() {
        assert!(build_lines("", None, 80).is_empty());
        assert!(build_lines("   ", None, 80).is_empty());
    }

    #[test]
    fn short_text_renders_a_five_row_block_with_ink() {
        let lines = build_lines("A", None, 80);
        assert_eq!(lines.len(), 5);
        assert!(lines.iter().any(|l| l.contains('█')));
    }

    #[test]
    fn sub_text_extends_the_block() {
        let with_sub = build_lines("A", Some("hi"), 80);
        assert!(with_sub.len() > 5);
    }

    #[test]
    fn oversized_text_is_trimmed_to_fit_the_grid() {
        // The real default-shaped failure: a long OS name that block-renders
        // far wider than a normal terminal.
        let long = "Fedora Linux 44 (Server Edition)";
        let lines = build_lines(long, None, 80);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        assert!(width <= 80, "art width {width} still exceeds the grid");
        assert!(lines.iter().any(|l| l.contains('█')));
    }

    #[test]
    fn impossible_width_yields_empty_art_rather_than_a_panic() {
        // A single block glyph is 5 columns wide, so a 2-column grid can
        // never show art. That must degrade to blank, not loop or panic.
        assert!(
            build_lines("Something", None, 2)
                .iter()
                .all(|l| l.trim().is_empty())
        );
    }

    #[test]
    fn parse_art_strips_padding_and_validates_bounds() {
        let art = "\n  ASCII\n  LOGO \n\n";
        let parsed = super::parse_art(art, None, 10, 5);
        assert!(parsed.is_some());
        let lines = parsed.unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "  ASCII");
        assert_eq!(lines[1], "  LOGO");

        // Fails when grid too small
        assert!(super::parse_art(art, None, 5, 5).is_none());
        assert!(super::parse_art(art, None, 10, 1).is_none());

        // Includes subtext when provided
        let with_sub = super::parse_art(art, Some("SUB"), 10, 5).unwrap();
        assert_eq!(with_sub.len(), 4);
        assert_eq!(with_sub[2], "");
        assert_eq!(with_sub[3], "SUB");
    }
}
