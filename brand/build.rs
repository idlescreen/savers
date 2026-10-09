// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use std::path::Path;

fn main() {
    let manifest = "libscreensaver_brand.idleplugin.toml";

    assert!(
        Path::new(manifest).exists(),
        "Required plugin manifest does not exist: {manifest}"
    );

    println!("cargo:rerun-if-changed={manifest}");

    // Embed brand icon when available on target platforms.
    build_support::embed_brand_icon("assets/scene-brand.ico", "idle-saver-brand");
}
