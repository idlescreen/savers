#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub use idle_api as runner;

/// Returns the IdleScreen plugin API version this saver was built against.
/// The host loader uses this for ABI negotiation; a mismatch is a hard
/// refusal (PluginError::ApiVersionMismatch). The function is `extern "C"`
/// so the host resolves it via `Library::get`.
#[unsafe(no_mangle)]
pub extern "C" fn idle_api_version() -> u32 {
    idle_api::API_VERSION
}

mod chaos;

// Re-exports for `chaos/benches/tick.rs`. See the matching
// note in `cosmos/src/lib.rs::bench_exports` for the rationale.
#[doc(hidden)]
pub mod bench_exports {
    pub use crate::chaos::Chaos;
}

#[cfg(test)]
mod tests_perf;

#[cfg(test)]
mod stress_tests;

#[unsafe(no_mangle)]
pub extern "C" fn create_screensaver() -> *mut idle_api::ScreensaverInstance {
    let effect = chaos::Chaos::new();
    let instance = idle_api::ScreensaverInstance {
        inner: Box::new(effect),
    };
    Box::into_raw(Box::new(instance))
}

/// Destroys a screensaver instance created by `create_screensaver`.
///
/// # Safety
///
/// The caller must ensure that `ptr` is a valid pointer allocated by `create_screensaver` and has not been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn destroy_screensaver(ptr: *mut idle_api::ScreensaverInstance) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[cfg(test)]
mod idle_api_version_tests {
    #[test]
    fn symbol_returns_host_api_version() {
        // The host loader resolves `idle_api_version` via dlsym; this
        // assertion pins that the saver advertises the same version the
        // host expects. A regression that drifts the saver (or builds
        // against a stale idle-api) would fail this test.
        assert_eq!(crate::idle_api_version(), idle_api::API_VERSION);
    }
}
