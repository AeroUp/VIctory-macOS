//! Probe for #31, on only with `VICTORY_PROBE_31=1`: no host command list
//! is being recorded while a submission runs, and no submission starts
//! while one is. Waits give up after two seconds (counted), so a thread
//! that keeps a list open while it submits another cannot hang a run.

use std::sync::{Condvar, Mutex, OnceLock};
use std::time::Duration;

/// Lists being recorded, and whether a submission is running.
static STATE: Mutex<(u32, bool)> = Mutex::new((0, false));
static CHANGED: Condvar = Condvar::new();
/// Waits that gave up.
pub(crate) static GAVE_UP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("VICTORY_PROBE_31").is_ok_and(|v| v == "1"))
}

fn wait_while(cond: impl Fn(&(u32, bool)) -> bool) -> std::sync::MutexGuard<'static, (u32, bool)> {
    let state = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let (state, timeout) = CHANGED
        .wait_timeout_while(state, Duration::from_secs(2), |s| cond(s))
        .unwrap_or_else(|e| e.into_inner());
    if timeout.timed_out() {
        let n = GAVE_UP.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        if n.is_power_of_two() {
            eprintln!("probe 31: a wait gave up after two seconds ({n} so far)");
        }
    }
    state
}

/// A list was opened for recording.
pub(crate) fn recording() {
    if on() {
        wait_while(|s| s.1).0 += 1;
    }
}

/// A list was closed.
pub(crate) fn recorded() {
    if on() {
        let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
        s.0 = s.0.saturating_sub(1);
        CHANGED.notify_all();
    }
}

/// Holds submission alone until dropped.
pub(crate) struct Submitting(bool);

pub(crate) fn submitting() -> Submitting {
    if !on() {
        return Submitting(false);
    }
    wait_while(|s| s.0 > 0 || s.1).1 = true;
    Submitting(true)
}

impl Drop for Submitting {
    fn drop(&mut self) {
        if self.0 {
            STATE.lock().unwrap_or_else(|e| e.into_inner()).1 = false;
            CHANGED.notify_all();
        }
    }
}
