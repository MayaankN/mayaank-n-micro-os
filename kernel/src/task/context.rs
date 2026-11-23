//! CPU Register Context Abstraction for Context Switching
//! Mayaank N - Micro-OS (25BTRCL148)

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    pub rsp: u64,
}

impl TaskContext {
    pub const fn empty() -> Self {
        Self { rsp: 0 }
    }

    /// Sets up an initial execution stack frame for a newly spawned task.
    ///
    /// The stack frame mimics a task paused inside `switch_context`:
    /// [stack_top - 8]:  Entry point (return address popped by `ret`)
    /// [stack_top - 16]: r15
    /// [stack_top - 24]: r14
    /// [stack_top - 32]: r13
    /// [stack_top - 40]: r12
    /// [stack_top - 48]: rbx
    /// [stack_top - 56]: rbp
    pub unsafe fn init_stack(&mut self, stack_top: usize, entry_point: usize) {
        // Ensure 16-byte alignment of top of stack
        let aligned_top = stack_top & !0xF;
        let stack = aligned_top as *mut u64;

        // Entry point is popped by `ret` instruction
        *stack.offset(-1) = entry_point as u64;

        // Callee-saved registers initialized to 0
        *stack.offset(-2) = 0; // r15
        *stack.offset(-3) = 0; // r14
        *stack.offset(-4) = 0; // r13
        *stack.offset(-5) = 0; // r12
        *stack.offset(-6) = 0; // rbx
        *stack.offset(-7) = 0; // rbp

        // Context pointer holds the top of this prepared frame
        self.rsp = stack.offset(-7) as u64;
    }
}
