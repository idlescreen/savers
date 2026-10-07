// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! C ABI parameter and telemetry control functions for WebAssembly.

use idle_api::{PALETTE_CALLBACK, ScreenPalette};
use std::sync::Mutex;

static DYNAMIC_ACCENT: Mutex<(u8, u8, u8)> = Mutex::new((120, 80, 255));
static DYNAMIC_DARK: Mutex<bool> = Mutex::new(true);

fn query_wasm_palette() -> ScreenPalette {
    let accent = match DYNAMIC_ACCENT.lock() {
        Ok(guard) => *guard,
        Err(poisoned) => *poisoned.into_inner(),
    };
    let dark_mode = match DYNAMIC_DARK.lock() {
        Ok(guard) => *guard,
        Err(poisoned) => *poisoned.into_inner(),
    };
    ScreenPalette::from_system(accent, dark_mode)
}

/// Ensures the dynamic palette callback is registered with `idle_api`.
pub fn ensure_palette_hook() {
    let _ = PALETTE_CALLBACK.set(query_wasm_palette);
}

/// Sets the live theme accent color from WebAssembly / JavaScript.
#[unsafe(no_mangle)]
pub extern "C" fn saver_set_accent(r: u8, g: u8, b: u8) {
    ensure_palette_hook();
    if let Ok(mut guard) = DYNAMIC_ACCENT.lock() {
        *guard = (r, g, b);
    }
}

/// Sets whether dark mode is preferred from WebAssembly / JavaScript.
#[unsafe(no_mangle)]
pub extern "C" fn saver_set_dark_mode(dark: bool) {
    ensure_palette_hook();
    if let Ok(mut guard) = DYNAMIC_DARK.lock() {
        *guard = dark;
    }
}

/// Feeds live audio bands (normalized 0.0..1.0) into the screensaver audio reactivity pipeline.
#[unsafe(no_mangle)]
pub extern "C" fn saver_set_audio_bands(bass: f32, low_mid: f32, mid: f32, treble: f32) {
    let bands = idle_api::audio::global_audio_bands();
    bands.set_band(0, bass);
    bands.set_band(1, low_mid);
    bands.set_band(2, mid);
    bands.set_band(3, treble);
}

/// Sets a runtime configuration parameter for screensavers (`key` = `val`).
///
/// # Safety
/// Pointers must point to valid UTF-8 strings of lengths `key_len` and `val_len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn saver_set_param(
    key_ptr: *const u8,
    key_len: usize,
    val_ptr: *const u8,
    val_len: usize,
) -> bool {
    if key_ptr.is_null() || val_ptr.is_null() {
        return false;
    }
    // SAFETY: caller guarantees pointer validity and length.
    let k_slice = unsafe { std::slice::from_raw_parts(key_ptr, key_len) };
    let v_slice = unsafe { std::slice::from_raw_parts(val_ptr, val_len) };

    let (Ok(k), Ok(v)) = (std::str::from_utf8(k_slice), std::str::from_utf8(v_slice)) else {
        return false;
    };

    if k == "text" || k == "logo_text" {
        idle_api::set_env("IDLE_LOGO_TEXT", v);
    }
    if let Some(env_k) = idle_api::saver_param_env_key(k) {
        idle_api::set_env(&env_k, v);
        true
    } else {
        k == "text" || k == "logo_text"
    }
}
