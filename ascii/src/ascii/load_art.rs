// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Resolves the art the saver animates.
//!
//! Input can be custom Omarchy branding, configured text, or session wordmark.

use crate::runner::{param, render_logo_block};

/// Reads Omarchy's branding file (`~/.config/omarchy/branding/screensaver.txt`) if present.
pub fn read_branding_file() -> Option<String> {
    let home = std::env::var_os("HOME")?;
    let path = std::path::PathBuf::from(home).join(".config/omarchy/branding/screensaver.txt");
    let content = std::fs::read_to_string(path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(content)
    }
}

/// `[saver] ascii.text`, falling back to the shared wordmark.
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

/// Resolve the full art string for ttfx.
///
/// Priority:
/// 1. `[saver] ascii.text` (rendered as block letters if fits cols, else plain).
/// 2. `crate::runner::logo()` (provided by daemon from branding file or logo_file).
/// 3. Omarchy branding file `~/.config/omarchy/branding/screensaver.txt` directly.
/// 4. Session wordmark (`crate::runner::wordmark()`), block-rendered.
pub fn resolve_art(cols: usize, rows: usize) -> String {
    let sub = resolve_sub_text();
    if let Some(text) = param("text") {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            let lines = build_lines(trimmed, sub.as_deref(), cols);
            if !lines.is_empty() {
                return lines.join("\n");
            }
            return trimmed.to_string();
        }
    }
    if let Some(art) = crate::runner::logo() {
        if !art.trim().is_empty() {
            return art.to_string();
        }
    }
    if let Some(branding) = read_branding_file() {
        return branding;
    }
    let lines = resolve_lines(cols, rows);
    if !lines.is_empty() {
        lines.join("\n")
    } else {
        resolve_text()
    }
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
        if let Some(branding) = read_branding_file() {
            if let Some(lines) = parse_art(&branding, sub.as_deref(), cols, rows) {
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

/// Pure art builder, trimming characters until the block fits `max_cols`.
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
        let long = "Fedora Linux 44 (Server Edition)";
        let lines = build_lines(long, None, 80);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        assert!(width <= 80, "art width {width} still exceeds the grid");
        assert!(lines.iter().any(|l| l.contains('█')));
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
    }
}
