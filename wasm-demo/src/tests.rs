// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use super::factory::create_saver_by_name;
use super::host::SaverHost;
use super::params::{
    saver_set_accent, saver_set_audio_bands, saver_set_dark_mode, saver_set_param,
};

#[test]
fn test_factory_creates_procedural_savers() {
    let names = [
        "aurora", "beams", "bursts", "chaos", "cosmos", "glyphs", "gnats", "hearth", "radar",
        "ripple", "storm",
    ];
    for name in names {
        let saver = create_saver_by_name(name);
        assert!(saver.is_some(), "failed to create saver: {name}");
    }
    assert!(create_saver_by_name("unknown_saver").is_none());
}

#[test]
fn test_saver_host_lifecycle() {
    let mut host = SaverHost::new_named("beams", 80, 24).expect("host creation");
    assert_eq!(host.cells_len(), 80 * 24 * 3);

    let ptr = host.tick(16.6);
    assert!(!ptr.is_null());

    host.resize(100, 30);
    assert_eq!(host.cells_len(), 100 * 30 * 3);
}

#[test]
fn test_params_control_api() {
    saver_set_accent(200, 100, 50);
    saver_set_dark_mode(false);
    saver_set_audio_bands(0.8, 0.4, 0.2, 0.1);

    let key = "glow";
    let val = "1.5";
    // SAFETY: pointers valid and length correct
    let ok = unsafe { saver_set_param(key.as_ptr(), key.len(), val.as_ptr(), val.len()) };
    assert!(ok);
}
