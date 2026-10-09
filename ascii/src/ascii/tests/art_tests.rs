// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use crate::ascii::load_art::{
    BrandTarget, build_lines, clean_kernel_version, clean_os_name, parse_art, resolve_target_text,
    visible_width,
};

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
    let width = lines.iter().map(|l| visible_width(l)).max().unwrap_or(0);
    assert!(width <= 80, "art width {width} still exceeds the grid");
    assert!(lines.iter().any(|l| l.contains('█')));
}

#[test]
fn parse_art_strips_padding_and_validates_bounds() {
    let art = "\n  ASCII\n  LOGO \n\n";
    let parsed = parse_art(art, None, 10, 5);
    assert!(parsed.is_some());
    let lines = parsed.unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "  ASCII");
    assert_eq!(lines[1], "  LOGO");
}

#[test]
fn parse_art_handles_ansi_color_sequences() {
    let colored = "\x1b[38;2;255;0;0mRED\x1b[0m\n\x1b[32mGREEN\x1b[0m";
    let parsed = parse_art(colored, None, 6, 2);
    assert!(parsed.is_some(), "ANSI escapes should not blow out width");
    assert_eq!(visible_width("\x1b[38;2;255;0;0mRED\x1b[0m"), 3);
}

#[test]
fn brand_target_rotation_cycles_through_os_de_kernel() {
    let t = BrandTarget::Os;
    let t = t.next();
    assert_eq!(t, BrandTarget::De);
    let t = t.next();
    assert_eq!(t, BrandTarget::Kernel);
    let t = t.next();
    assert_eq!(t, BrandTarget::Os);
}

#[test]
fn brand_target_toggle_matches_next() {
    assert_eq!(BrandTarget::Os.toggle(), BrandTarget::De);
    assert_eq!(BrandTarget::De.toggle(), BrandTarget::Kernel);
    assert_eq!(BrandTarget::Kernel.toggle(), BrandTarget::Os);
}

#[test]
fn resolve_target_text_returns_non_empty_for_all_targets() {
    assert!(!resolve_target_text(BrandTarget::Os).is_empty());
    assert!(!resolve_target_text(BrandTarget::De).is_empty());
    let kernel_text = resolve_target_text(BrandTarget::Kernel);
    assert!(!kernel_text.is_empty());
    assert!(kernel_text.contains("Linux") || kernel_text.chars().any(|c| c.is_ascii_digit()));
}

#[test]
fn clean_os_name_simplifies_distro_editions() {
    assert_eq!(
        clean_os_name("Fedora Linux 44 (Server Edition)"),
        "Fedora Linux"
    );
    assert_eq!(clean_os_name("Fedora Linux"), "Fedora Linux");
    assert_eq!(clean_os_name("Arch Linux"), "Arch Linux");
    assert_eq!(clean_os_name("Fedora"), "Fedora");
}

#[test]
fn clean_kernel_version_formats_clean_release_without_build_noise() {
    assert_eq!(clean_kernel_version("7.2.9-200.fc44.x86_64"), "Linux 7.2.9");
    assert_eq!(clean_kernel_version("6.13.2-arch1-1"), "Linux 6.13.2");
    assert_eq!(clean_kernel_version("6.8.0-45-generic"), "Linux 6.8.0");
    assert_eq!(clean_kernel_version("Linux 6.13"), "Linux 6.13");
    assert_eq!(clean_kernel_version("unknown"), "Linux");
    assert_eq!(clean_kernel_version(""), "Linux");
}
