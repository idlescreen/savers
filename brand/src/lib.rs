// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Dedicated brand screensaver plugin.
//!
//! Renders the dynamically detected host OS or active desktop environment
//! with terminal text effects.

pub use idle_api as runner;

/// Returns the IdleScreen plugin API version this saver was built against.
#[cfg_attr(not(feature = "bundled"), unsafe(no_mangle))]
pub extern "C" fn idle_api_version() -> u32 {
    idle_api::API_VERSION
}

/// Allocates a new brand screensaver instance.
#[cfg_attr(not(feature = "bundled"), unsafe(no_mangle))]
pub extern "C" fn create_screensaver() -> *mut idle_api::ScreensaverInstance {
    let instance = idle_api::ScreensaverInstance {
        inner: Box::new(screensaver_ascii::bench_exports::Ascii::new()),
    };
    Box::into_raw(Box::new(instance))
}

/// Destroys a screensaver instance created by [`create_screensaver`].
///
/// # Safety
///
/// The caller must ensure that `ptr` is a valid pointer allocated by
/// [`create_screensaver`] and has not yet been freed.
#[cfg_attr(not(feature = "bundled"), unsafe(no_mangle))]
pub unsafe extern "C" fn destroy_screensaver(ptr: *mut idle_api::ScreensaverInstance) {
    if !ptr.is_null() {
        // SAFETY: `ptr` was allocated by Box::into_raw in `create_screensaver`.
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn brand_saver_creates_instance() {
        let ptr = crate::create_screensaver();
        assert!(!ptr.is_null());
        unsafe {
            crate::destroy_screensaver(ptr);
        }
    }

    #[test]
    fn brand_api_version_matches_host() {
        assert_eq!(crate::idle_api_version(), idle_api::API_VERSION);
    }
}
