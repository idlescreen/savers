// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use std::path::Path;

fn main() {
    let manifest = "libscreensaver_ascii.idleplugin.toml";

    assert!(
        Path::new(manifest).exists(),
        "Required plugin manifest does not exist: {manifest}"
    );

    println!("cargo:rerun-if-changed={manifest}");

    // No brand icon yet — `embed_brand_icon` no-ops when the file is absent
    // and on non-Windows targets, so the build succeeds either way. Pass the
    // conventional path so dropping an .ico in later needs no build.rs change.
    build_support::embed_brand_icon("assets/scene-ascii.ico", "idle-saver-ascii");
}