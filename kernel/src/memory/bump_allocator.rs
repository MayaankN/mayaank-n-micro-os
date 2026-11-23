//! Bump Allocator for Bare-Metal Kernel Heap
//! Mayaank N - Micro-OS (25BTRCL148)

pub struct BumpAllocator {
    heap_start: usize,
    heap_size: usize,
    next: usize,
    allocations: usize,
    allocated_bytes: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_size: 0,
            next: 0,
            allocations: 0,
            allocated_bytes: 0,
        }
    }

    /// Initializes the bump allocator with a dedicated physical/virtual memory region.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_size = heap_size;
        self.next = heap_start;
        self.allocations = 0;
        self.allocated_bytes = 0;
    }

    /// Allocates memory of the specified size and byte alignment.
    pub fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        let current_next = self.next;
        // Align up the pointer
        let aligned_next = align_up(current_next, align);
        let end = aligned_next.checked_add(size)?;

        if end <= self.heap_start + self.heap_size {
            self.next = end;
            self.allocations += 1;
            self.allocated_bytes += size;
            Some(aligned_next as *mut u8)
        } else {
            None // Out of memory
        }
    }

    pub fn total_capacity(&self) -> usize {
        self.heap_size
    }

    pub fn used_bytes(&self) -> usize {
        self.allocated_bytes
    }

    pub fn remaining_bytes(&self) -> usize {
        if self.heap_size >= self.allocated_bytes {
            self.heap_size - self.allocated_bytes
        } else {
            0
        }
    }

    pub fn allocation_count(&self) -> usize {
        self.allocations
    }

    /// Resets the bump allocator if all allocations are discarded.
    pub unsafe fn reset(&mut self) {
        self.next = self.heap_start;
        self.allocations = 0;
        self.allocated_bytes = 0;
    }
}

/// Aligns a memory address up to the nearest multiple of align.
#[inline]
pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
