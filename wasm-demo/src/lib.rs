//! Browser host shim — runs an IdleScreen saver inside wasm32.
//!
//! Plain C ABI, no wasm-bindgen: JS drives time via rAF and reads the
//! packed cell buffer directly out of wasm linear memory.
//!
//! Packed layout, three u32 words per cell:
//!   [codepoint, fg_r | fg_g<<8 | fg_b<<16 | bold<<24, bg_r | bg_g<<8 | bg_b<<16]

use std::time::Duration;

use idle_api::{ScreensaverInstance, TerminalCell};

/// Owns the saver instance plus its render grid and packed output buffer.
pub struct SaverHost {
    inst: Box<ScreensaverInstance>,
    grid: Vec<TerminalCell>,
    packed: Vec<u32>,
    cols: usize,
    rows: usize,
}

/// Creates a host running `beams` at the given grid size.
/// Returns null on failure.
#[unsafe(no_mangle)]
pub extern "C" fn saver_new(cols: usize, rows: usize) -> *mut SaverHost {
    if cols == 0 || rows == 0 || cols > 512 || rows > 256 {
        return std::ptr::null_mut();
    }
    let raw = screensaver_beams::create_screensaver();
    if raw.is_null() {
        return std::ptr::null_mut();
    }
    let mut inst = unsafe { Box::from_raw(raw) };
    inst.inner.init(cols, rows);
    inst.inner.set_active(true);
    inst.inner.set_focused(true);
    Box::into_raw(Box::new(SaverHost {
        inst,
        grid: vec![TerminalCell::default(); cols * rows],
        packed: vec![0u32; cols * rows * 3],
        cols,
        rows,
    }))
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
    let dt = Duration::from_secs_f64((dt_ms / 1000.0).max(0.0));
    h.inst.inner.update(dt, h.cols, h.rows);
    h.inst.inner.update_frame_time(dt);
    h.inst.inner.draw(&mut h.grid, h.cols, h.rows);
    for (i, c) in h.grid.iter().enumerate() {
        h.packed[i * 3] = c.ch as u32;
        h.packed[i * 3 + 1] = (c.fg.0 as u32)
            | (c.fg.1 as u32) << 8
            | (c.fg.2 as u32) << 16
            | (u32::from(c.bold) << 24);
        h.packed[i * 3 + 2] = (c.bg.0 as u32) | (c.bg.1 as u32) << 8 | (c.bg.2 as u32) << 16;
    }
    h.packed.as_ptr()
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
    unsafe { (*host).packed.len() }
}

/// Rebuilds the grid at a new size (canvas resized).
///
/// # Safety
/// `host` must be a pointer returned by `saver_new`, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn saver_resize(host: *mut SaverHost, cols: usize, rows: usize) {
    if host.is_null() || cols == 0 || rows == 0 || cols > 512 || rows > 256 {
        return;
    }
    let h = unsafe { &mut *host };
    h.cols = cols;
    h.rows = rows;
    h.grid = vec![TerminalCell::default(); cols * rows];
    h.packed = vec![0u32; cols * rows * 3];
    h.inst.inner.init(cols, rows);
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
