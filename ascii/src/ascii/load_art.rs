// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Resolves the art the saver animates.
//!
//! Input can be custom Omarchy branding, configured text, or session wordmark.

use crate::runner::{param, render_logo_block};

/// Reads Omarchy's branding file (`~/.config/omarchy/branding/screensaver.txt`) if present.
#[allow(dead_code)]
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

/// The alternating brand targets for rotation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrandTarget {
    Os,
    De,
    Kernel,
}

impl BrandTarget {
    pub fn random_excluding(rng: &mut crate::runner::LcgRng, exclude: Option<BrandTarget>) -> Self {
        match exclude {
            Some(BrandTarget::Os) => match (rng.next_u64() >> 33) & 1 {
                0 => BrandTarget::De,
                _ => BrandTarget::Kernel,
            },
            Some(BrandTarget::De) => match (rng.next_u64() >> 33) & 1 {
                0 => BrandTarget::Os,
                _ => BrandTarget::Kernel,
            },
            Some(BrandTarget::Kernel) => match (rng.next_u64() >> 33) & 1 {
                0 => BrandTarget::Os,
                _ => BrandTarget::De,
            },
            None => match (rng.next_u64() >> 33) % 3 {
                0 => BrandTarget::Os,
                1 => BrandTarget::De,
                _ => BrandTarget::Kernel,
            },
        }
    }

    pub fn next(self) -> Self {
        match self {
            BrandTarget::Os => BrandTarget::De,
            BrandTarget::De => BrandTarget::Kernel,
            BrandTarget::Kernel => BrandTarget::Os,
        }
    }

    #[allow(dead_code)]
    pub fn toggle(self) -> Self {
        self.next()
    }
}

/// Explicit user text override from `[saver] ascii.text` or aliases.
pub fn custom_text() -> Option<String> {
    param("ascii.text")
        .or_else(|| param("brand.text"))
        .or_else(|| param("text"))
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

/// Cleans raw Host OS text (e.g. "Fedora Linux 44 (Server Edition)") into clean brand text (e.g. "Fedora Linux").
pub fn clean_os_name(raw: &str) -> String {
    let s = raw.split(" (").next().unwrap_or(raw).trim();
    if let Some(pos) = s.rfind(' ') {
        let suffix = &s[pos + 1..];
        if suffix.chars().all(|c| c.is_ascii_digit() || c == '.') && !s[..pos].trim().is_empty() {
            return s[..pos].trim().to_string();
        }
    }
    s.to_string()
}

/// Cleans raw kernel version string (e.g. "7.2.9-200.fc44.x86_64") into clean release text (e.g. "Linux 7.2.9").
pub fn clean_kernel_version(raw: &str) -> String {
    let trimmed = raw.trim();
    let stripped = trimmed
        .strip_prefix("Linux ")
        .or_else(|| trimmed.strip_prefix("linux "))
        .unwrap_or(trimmed);
    let ver = stripped.split('-').next().unwrap_or(stripped).trim();
    if ver.is_empty() || ver == "unknown" {
        "Linux".to_string()
    } else {
        format!("Linux {ver}")
    }
}

/// Resolves text for the specified brand target (`Os`, `De`, or `Kernel`), respecting user overrides.
pub fn resolve_target_text(target: BrandTarget) -> String {
    if let Some(custom) = custom_text() {
        return custom;
    }
    match target {
        BrandTarget::Os => {
            let os = crate::runner::detect_host_os().unwrap_or_else(|| "Linux".into());
            clean_os_name(&os)
        }
        BrandTarget::De => {
            crate::runner::detect_desktop_environment().unwrap_or_else(|| "IDLESCREEN".into())
        }
        BrandTarget::Kernel => {
            let k = crate::runner::detect_kernel().unwrap_or_else(|| "Linux".into());
            clean_kernel_version(&k)
        }
    }
}

#[allow(dead_code)]
pub fn resolve_text() -> String {
    resolve_target_text(BrandTarget::Os)
}

/// Optional second line beneath the art block.
pub fn resolve_sub_text() -> Option<String> {
    param("sub")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Visible character width of a string, ignoring ANSI terminal escape codes.
pub fn visible_width(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut count = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b {
            i += 1;
            if i < bytes.len() && bytes[i] == b'[' {
                i += 1;
                while i < bytes.len() && (bytes[i] < b'@' || bytes[i] > b'~') {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
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
        i = (i + len).min(bytes.len());
        count += 1;
    }
    count
}

/// Resolve the full art string for ttfx targeting the specified brand.
pub fn resolve_art_for_target(target: BrandTarget, cols: usize, rows: usize) -> String {
    let lines = resolve_lines_for_target(target, cols, rows);
    if !lines.is_empty() {
        lines.join("\n")
    } else {
        resolve_target_text(target)
    }
}

/// Resolve the full art string for ttfx.
#[allow(dead_code)]
pub fn resolve_art(cols: usize, rows: usize) -> String {
    resolve_art_for_target(BrandTarget::Os, cols, rows)
}

/// Render the target text or custom art, sized to fit a `cols`×`rows` grid.
pub fn resolve_lines_for_target(target: BrandTarget, cols: usize, _rows: usize) -> Vec<String> {
    let sub = resolve_sub_text();
    if let Some(custom) = custom_text() {
        let lines = build_lines(&custom, sub.as_deref(), cols);
        if !lines.is_empty() {
            return lines;
        }
    }
    build_lines(&resolve_target_text(target), sub.as_deref(), cols)
}

/// Render the resolved text or custom art, sized to fit a `cols`×`rows` grid.
#[allow(dead_code)]
pub fn resolve_lines(cols: usize, rows: usize) -> Vec<String> {
    resolve_lines_for_target(BrandTarget::Os, cols, rows)
}

/// Pure custom art parser, validating dimensions against the grid.
#[allow(dead_code)]
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
    let width = lines.iter().map(|l| visible_width(l)).max().unwrap_or(0);
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
        let width = lines.iter().map(|l| visible_width(l)).max().unwrap_or(0);
        if width <= max_cols || chars.is_empty() {
            return lines;
        }
        chars.pop();
    }
}
