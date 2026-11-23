//! Bump Allocator Model for Verification
//! Mayaank N - Micro-OS (25BTRCL148)

#[derive(Debug)]
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

    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_size = heap_size;
        self.next = heap_start;
        self.allocations = 0;
        self.allocated_bytes = 0;
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> Option<usize> {
        let aligned_next = align_up(self.next, align);
        let end = aligned_next.checked_add(size)?;

        if end <= self.heap_start + self.heap_size {
            self.next = end;
            self.allocations += 1;
            self.allocated_bytes += size;
            Some(aligned_next)
        } else {
            None
        }
    }

    pub fn total_capacity(&self) -> usize {
        self.heap_size
    }

    pub fn used_bytes(&self) -> usize {
        self.allocated_bytes
    }

    pub fn remaining_bytes(&self) -> usize {
        self.heap_size.saturating_sub(self.allocated_bytes)
    }

    pub fn allocation_count(&self) -> usize {
        self.allocations
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
