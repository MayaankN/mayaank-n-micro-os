//! Synchronization Primitives
//! Mayaank N - Micro-OS (25BTRCL148)

pub mod spinlock;

pub use spinlock::{SpinLock, SpinLockGuard};
