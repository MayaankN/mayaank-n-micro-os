//! Kernel Heap Initialization and Global Tracking
//! Mayaank N - Micro-OS (25BTRCL148)

use core::cell::UnsafeCell;
use crate::memory::bump_allocator::BumpAllocator;
use crate::sync::SpinLock;

pub const KERNEL_HEAP_SIZE: usize = 2 * 1024 * 1024; // 2 MiB Kernel Heap

#[repr(align(4096))]
#[allow(dead_code)]
struct HeapBuffer([u8; KERNEL_HEAP_SIZE]);

struct SyncHeapBuffer(UnsafeCell<HeapBuffer>);
unsafe impl Sync for SyncHeapBuffer {}

static HEAP_BUFFER: SyncHeapBuffer = SyncHeapBuffer(UnsafeCell::new(HeapBuffer([0; KERNEL_HEAP_SIZE])));

pub static KERNEL_ALLOCATOR: SpinLock<BumpAllocator> = SpinLock::new(BumpAllocator::new());

/// Initializes the kernel heap bump allocator.
pub fn init_heap() {
    let heap_start = HEAP_BUFFER.0.get() as usize;
    unsafe {
        KERNEL_ALLOCATOR.lock_irq_disabled().init(heap_start, KERNEL_HEAP_SIZE);
    }
}

/// Allocates aligned bytes from the kernel heap.
pub fn kmalloc(size: usize, align: usize) -> Option<*mut u8> {
    KERNEL_ALLOCATOR.lock_irq_disabled().allocate(size, align)
}
