// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Screensaver factory resolving saver instances by name.

use idle_api::ScreensaverInstance;

/// Instantiates a screensaver instance by its canonical organizing name.
pub fn create_saver_by_name(name: &str) -> Option<Box<ScreensaverInstance>> {
    let raw = match name {
        "aurora" => screensaver_aurora::create_screensaver(),
        "beams" => screensaver_beams::create_screensaver(),
        "bursts" => screensaver_bursts::create_screensaver(),
        "chaos" => screensaver_chaos::create_screensaver(),
        "cosmos" => screensaver_cosmos::create_screensaver(),
        "glyphs" => screensaver_glyphs::create_screensaver(),
        "gnats" => screensaver_gnats::create_screensaver(),
        "hearth" => screensaver_hearth::create_screensaver(),
        "radar" => screensaver_radar::create_screensaver(),
        "ripple" => screensaver_ripple::create_screensaver(),
        "storm" => screensaver_storm::create_screensaver(),
        _ => return None,
    };

    if raw.is_null() {
        None
    } else {
        // SAFETY: `raw` was allocated by the corresponding saver's create_screensaver.
        Some(unsafe { Box::from_raw(raw) })
    }
}

/// Creates a screensaver instance by name via C ABI.
///
/// # Safety
/// `name_ptr` must point to valid UTF-8 of length `name_len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn saver_create_named(
    name_ptr: *const u8,
    name_len: usize,
) -> *mut ScreensaverInstance {
    if name_ptr.is_null() || name_len == 0 {
        return std::ptr::null_mut();
    }
    // SAFETY: caller guarantees pointer validity and length.
    let slice = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
    let Ok(name) = std::str::from_utf8(slice) else {
        return std::ptr::null_mut();
    };

    match create_saver_by_name(name) {
        Some(inst) => Box::into_raw(inst),
        None => std::ptr::null_mut(),
    }
}
