//! Memory Management Subsystem
//! Mayaank N - Micro-OS (25BTRCL148)

pub mod bump_allocator;
pub mod heap;

pub use bump_allocator::{align_up, BumpAllocator};
pub use heap::{init_heap, kmalloc, KERNEL_ALLOCATOR, KERNEL_HEAP_SIZE};
