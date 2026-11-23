//! System Timer and Hardware Tick Tracking
//! Mayaank N - Micro-OS (25BTRCL148)

use core::sync::atomic::{AtomicU64, Ordering};

static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);

/// Returns the elapsed timer ticks since kernel boot.
#[inline]
pub fn current_ticks() -> u64 {
    SYSTEM_TICKS.load(Ordering::Relaxed)
}

/// Increments the system tick counter (invoked by timer ISR).
#[inline]
pub fn increment_ticks() -> u64 {
    SYSTEM_TICKS.fetch_add(1, Ordering::SeqCst) + 1
}
