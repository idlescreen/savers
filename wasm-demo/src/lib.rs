// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Browser host shim — runs an IdleScreen saver inside wasm32.
//!
//! Plain C ABI, no wasm-bindgen: JS drives time via rAF and reads the
//! packed cell buffer directly out of wasm linear memory.
//!
//! Packed layout, three u32 words per cell:
//!   [codepoint, fg_r | fg_g<<8 | fg_b<<16 | bold<<24, bg_r | bg_g<<8 | bg_b<<16]

pub mod factory;
pub mod host;
pub mod params;
#[cfg(test)]
mod tests;

pub use factory::create_saver_by_name;
pub use host::SaverHost;
pub use params::{saver_set_accent, saver_set_audio_bands, saver_set_dark_mode, saver_set_param};

/// Creates a host running `beams` at the given grid size.
/// Returns null on failure.
#[unsafe(no_mangle)]
pub extern "C" fn saver_new(cols: usize, rows: usize) -> *mut SaverHost {
    match SaverHost::new(cols, rows) {
        Some(h) => Box::into_raw(h),
        None => std::ptr::null_mut(),
    }
}

/// Creates a host running a named screensaver at the given grid size.
///
/// # Safety
/// `name_ptr` must point to a valid UTF-8 string of length `name_len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn saver_new_named(
    name_ptr: *const u8,
    name_len: usize,
    cols: usize,
    rows: usize,
) -> *mut SaverHost {
    if name_ptr.is_null() || name_len == 0 {
        return std::ptr::null_mut();
    }
    // SAFETY: caller guarantees pointer validity and length.
    let slice = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
    let Ok(name) = std::str::from_utf8(slice) else {
        return std::ptr::null_mut();
    };
    match SaverHost::new_named(name, cols, rows) {
        Some(h) => Box::into_raw(h),
        None => std::ptr::null_mut(),
    }
}

/// Reserves `len` bytes in linear memory for the caller to write a saver name
/// into, and returns the offset. Pass it to [`saver_new_named`].
///
/// JavaScript has no way to hand a wasm module a string without copying it
/// into linear memory first, so these two exports exist purely for that.
#[unsafe(no_mangle)]
pub extern "C" fn saver_alloc(len: usize) -> *mut u8 {
    let mut buf: Vec<u8> = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf); // Leaked deliberately: freed by module teardown.
    ptr
}

/// Advances the saver by `dt_ms` milliseconds, redraws the grid, and
/// returns a pointer to the packed cell buffer (valid until next call).
///
/// # Safety
/// `host` must be a pointer returned by `saver_new` or `saver_new_named`, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn saver_tick(host: *mut SaverHost, dt_ms: f64) -> *const u32 {
    if host.is_null() {
        return std::ptr::null();
    }
    // SAFETY: caller guarantees host pointer validity.
    let h = unsafe { &mut *host };
    h.tick(dt_ms)
}

/// Number of u32 words in the packed buffer (cols * rows * 3).
///
/// # Safety
/// `host` must be a pointer returned by `saver_new` or `saver_new_named`, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn saver_cells_len(host: *const SaverHost) -> usize {
    if host.is_null() {
        return 0;
    }
    // SAFETY: caller guarantees host pointer validity.
    unsafe { (*host).cells_len() }
}

/// Rebuilds the grid at a new size (canvas resized).
///
/// # Safety
/// `host` must be a pointer returned by `saver_new` or `saver_new_named`, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn saver_resize(host: *mut SaverHost, cols: usize, rows: usize) {
    if host.is_null() {
        return;
    }
    // SAFETY: caller guarantees host pointer validity.
    let h = unsafe { &mut *host };
    h.resize(cols, rows);
}

/// Frees a host created by `saver_new` or `saver_new_named`.
///
/// # Safety
/// `host` must be a pointer returned by `saver_new` or `saver_new_named`, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn saver_destroy(host: *mut SaverHost) {
    if !host.is_null() {
        // SAFETY: caller guarantees host is valid and transfers ownership.
        unsafe {
            drop(Box::from_raw(host));
        }
    }
}
