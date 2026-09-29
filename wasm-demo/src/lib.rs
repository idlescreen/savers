//! Browser host shim — runs an IdleScreen saver inside wasm32.
//!
//! Plain C ABI, no wasm-bindgen: JS drives time via rAF and reads the
//! packed cell buffer directly out of wasm linear memory.
//!
//! Packed layout, three u32 words per cell:
//!   [codepoint, fg_r | fg_g<<8 | fg_b<<16 | bold<<24, bg_r | bg_g<<8 | bg_b<<16]

pub mod host;
pub use host::SaverHost;

/// Creates a host running `beams` at the given grid size.
/// Returns null on failure.
#[unsafe(no_mangle)]
pub extern "C" fn saver_new(cols: usize, rows: usize) -> *mut SaverHost {
    match SaverHost::new(cols, rows) {
        Some(h) => Box::into_raw(h),
        None => std::ptr::null_mut(),
    }
}

/// Advances the saver by `dt_ms` milliseconds, redraws the grid, and
/// returns a pointer to the packed cell buffer (valid until next call).
///
/// # Safety
/// `host` must be a pointer returned by `saver_new`, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn saver_tick(host: *mut SaverHost, dt_ms: f64) -> *const u32 {
    if host.is_null() {
        return std::ptr::null();
    }
    let h = unsafe { &mut *host };
    h.tick(dt_ms)
}

/// Number of u32 words in the packed buffer (cols * rows * 3).
///
/// # Safety
/// `host` must be a pointer returned by `saver_new`, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn saver_cells_len(host: *const SaverHost) -> usize {
    if host.is_null() {
        return 0;
    }
    unsafe { (*host).cells_len() }
}

/// Rebuilds the grid at a new size (canvas resized).
///
/// # Safety
/// `host` must be a pointer returned by `saver_new`, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn saver_resize(host: *mut SaverHost, cols: usize, rows: usize) {
    if host.is_null() {
        return;
    }
    let h = unsafe { &mut *host };
    h.resize(cols, rows);
}

/// Frees a host created by `saver_new`.
///
/// # Safety
/// `host` must be a pointer returned by `saver_new`, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn saver_destroy(host: *mut SaverHost) {
    if !host.is_null() {
        unsafe {
            drop(Box::from_raw(host));
        }
    }
}
