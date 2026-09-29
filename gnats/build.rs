// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use std::path::Path;

fn main() {
    let icon = "assets/scene-gnats.ico";
    let manifest = "libscreensaver_gnats.idleplugin.toml";

    assert!(
        Path::new(icon).exists(),
        "Required icon asset does not exist: {icon}"
    );
    assert!(
        Path::new(manifest).exists(),
        "Required plugin manifest does not exist: {manifest}"
    );

    println!("cargo:rerun-if-changed={icon}");
    println!("cargo:rerun-if-changed={manifest}");

    build_support::embed_brand_icon(icon, "idle-saver-gnats");
}
