// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Character ramp tables, loaded from `assets/ramps.txt` at init.
//!
//! Static art data lives in a dedicated file rather than inline literals, per
//! the `AGENTS.md` rule on blobs. Parsing happens once; effects then hold a
//! borrowed slice and never re-parse.

/// `name<whitespace>ramp-chars`, one ramp per line.
static RAMP_TABLE: &str = include_str!("../../assets/ramps.txt");

/// Fallback used when a named ramp is absent from the table.
const DEFAULT_NAME: &str = "blocks";

pub struct Ramps {
    blocks: Vec<char>,
    matrix: Vec<char>,
    minimal: Vec<char>,
}

impl Default for Ramps {
    fn default() -> Self {
        Self::load()
    }
}

impl Ramps {
    /// Parse the ramp table. Unknown or malformed lines are skipped; the
    /// caller falls back to [`Ramps::pick`].
    pub fn load() -> Self {
        let mut blocks = Vec::new();
        let mut matrix = Vec::new();
        let mut minimal = Vec::new();

        for line in RAMP_TABLE.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.splitn(2, char::is_whitespace);
            let name = parts.next().unwrap_or_default();
            let chars: Vec<char> = parts.next().unwrap_or_default().chars().collect();
            if chars.is_empty() {
                continue;
            }
            match name {
                "blocks" => blocks = chars,
                "matrix" => matrix = chars,
                "minimal" => minimal = chars,
                _ => {}
            }
        }

        if blocks.is_empty() {
            blocks = vec!['#', '+', '-', '.', ' '];
        }
        if matrix.is_empty() {
            matrix = blocks.clone();
        }
        if minimal.is_empty() {
            minimal = vec!['.', ':', '-', '=', '+', '*', '#', '%', '@'];
        }
        Self {
            blocks,
            matrix,
            minimal,
        }
    }

    /// Look up a ramp by `[saver] ascii.ramp` value, falling back to the
    /// default ramp for unknown names.
    pub fn pick(&self, name: &str) -> &[char] {
        match name {
            "matrix" => &self.matrix,
            "minimal" => &self.minimal,
            _ => &self.blocks,
        }
    }

    pub fn default_name() -> &'static str {
        DEFAULT_NAME
    }
}

#[cfg(test)]
mod ramp_tests {
    use super::Ramps;

    #[test]
    fn every_named_ramp_resolves_to_characters() {
        let r = Ramps::load();
        assert!(!r.pick("blocks").is_empty());
        assert!(!r.pick("matrix").is_empty());
        assert!(!r.pick("minimal").is_empty());
    }

    #[test]
    fn unknown_name_falls_back_to_the_default_ramp() {
        let r = Ramps::load();
        assert_eq!(r.pick("nonsense"), r.pick(Ramps::default_name()));
    }

    #[test]
    fn table_is_not_empty_on_disk() {
        assert!(!super::RAMP_TABLE.trim().is_empty());
    }
}
