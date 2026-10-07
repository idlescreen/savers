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

/// Render the resolved text, sized to fit a `cols`-wide grid.
pub fn resolve_lines(cols: usize) -> Vec<String> {
    let sub = resolve_sub_text();
    build_lines(&resolve_text(), sub.as_deref(), cols)
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
}
