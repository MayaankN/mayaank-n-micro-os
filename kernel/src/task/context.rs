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
    /// [stack_top - 16]: rbp
    /// [stack_top - 24]: rbx
    /// [stack_top - 32]: r12
    /// [stack_top - 40]: r13
    /// [stack_top - 48]: r14
    /// [stack_top - 56]: r15 (top of initial context, popped first by switch_context)
    pub unsafe fn init_stack(&mut self, stack_top: usize, entry_point: usize) {
        // Ensure 16-byte alignment of top of stack
        let aligned_top = stack_top & !0xF;
        let stack = aligned_top as *mut u64;

        // Entry point is popped by `ret` instruction
        *stack.offset(-1) = entry_point as u64;

        // Callee-saved registers initialized to 0 (popped in order: r15, r14, r13, r12, rbx, rbp)
        *stack.offset(-2) = 0; // rbp (popped 6th)
        *stack.offset(-3) = 0; // rbx (popped 5th)
        *stack.offset(-4) = 0; // r12 (popped 4th)
        *stack.offset(-5) = 0; // r13 (popped 3rd)
        *stack.offset(-6) = 0; // r14 (popped 2nd)
        *stack.offset(-7) = 0; // r15 (popped 1st)

        // Context pointer holds the top of this prepared frame
        self.rsp = stack.offset(-7) as u64;
    }
}
