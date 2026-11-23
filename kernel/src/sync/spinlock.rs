//! Bare-Metal Spinlock Implementation
//! Mayaank N - Micro-OS (25BTRCL148)

use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// A hardware-level atomic spinlock protecting shared kernel resources.
pub struct SpinLock<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Sync for SpinLock<T> {}
unsafe impl<T: ?Sized + Send> Send for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// Creates a new unlocked SpinLock wrapping data.
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> SpinLock<T> {
    /// Acquires the lock, spinning until it is available.
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self.lock.swap(true, Ordering::Acquire) {
            while self.lock.load(Ordering::Relaxed) {
                spin_loop();
            }
        }
        SpinLockGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
            rflags: None,
        }
    }

    /// Attempts to acquire the lock without blocking.
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        if !self.lock.swap(true, Ordering::Acquire) {
            Some(SpinLockGuard {
                lock: &self.lock,
                data: unsafe { &mut *self.data.get() },
                rflags: None,
            })
        } else {
            None
        }
    }

    /// Acquires the lock with interrupts disabled to prevent deadlocks in ISRs.
    pub fn lock_irq_disabled(&self) -> SpinLockGuard<'_, T> {
        let rflags = read_and_disable_interrupts();
        while self.lock.swap(true, Ordering::Acquire) {
            while self.lock.load(Ordering::Relaxed) {
                spin_loop();
            }
        }
        SpinLockGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
            rflags: Some(rflags),
        }
    }

    /// Unconditionally forces an unlock (for emergency panic or bootstrap transitions).
    pub unsafe fn force_unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
}

/// An RAII scoped guard holding access to the spinlock-protected data.
pub struct SpinLockGuard<'a, T: ?Sized> {
    lock: &'a AtomicBool,
    data: &'a mut T,
    rflags: Option<u64>,
}

impl<'a, T: ?Sized> Deref for SpinLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: ?Sized> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T: ?Sized> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
        if let Some(flags) = self.rflags {
            restore_interrupts(flags);
        }
    }
}

#[inline]
fn read_and_disable_interrupts() -> u64 {
    let rflags: u64;
    unsafe {
        core::arch::asm!(
            "pushfq",
            "pop {0}",
            "cli",
            out(reg) rflags,
            options(nomem, preserves_flags)
        );
    }
    rflags
}

#[inline]
fn restore_interrupts(rflags: u64) {
    if (rflags & 0x200) != 0 {
        unsafe {
            core::arch::asm!("sti", options(nomem, nostack));
        }
    }
}
