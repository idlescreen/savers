use std::sync::Mutex;

mod behavior;
mod lifecycle;
mod light;

/// Process-global env races under parallel tests.
/// Host reads `IDLE_SECONDARY_MONITOR` (see idle-runner sys_info).
static MONITOR_ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_primary_monitor<R>(f: impl FnOnce() -> R) -> R {
    let _g = MONITOR_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        std::env::remove_var("IDLE_SECONDARY_MONITOR");
        std::env::remove_var("TRANCE_SECONDARY_MONITOR");
    }
    f()
}

fn with_secondary_monitor<R>(f: impl FnOnce() -> R) -> R {
    let _g = MONITOR_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        std::env::set_var("IDLE_SECONDARY_MONITOR", "1");
    }
    let out = f();
    unsafe {
        std::env::remove_var("IDLE_SECONDARY_MONITOR");
    }
    out
}
