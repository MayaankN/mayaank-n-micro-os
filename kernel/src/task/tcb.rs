//! Task Control Block (TCB) Implementation
//! Mayaank N - Micro-OS (25BTRCL148)

use crate::task::context::TaskContext;
use crate::task::state::TaskState;
use crate::task::task_id::TaskId;

pub const DEFAULT_STACK_SIZE: usize = 32 * 1024; // 32 KiB Stack per task

pub struct TaskControlBlock {
    pub id: TaskId,
    pub name: &'static str,
    pub state: TaskState,
    pub context: TaskContext,
    pub stack_base: usize,
    pub stack_size: usize,
    pub priority: u8,
    pub assigned_quantum: usize,
    pub quantum_remaining: usize,
    pub total_cpu_ticks: u64,
    pub current_burst_ticks: usize,
    pub burst_history: [usize; 4],
    pub burst_count: usize,
    pub created_at_tick: u64,
    pub context_switches: u64,
    pub total_wait_ticks: u64,
    pub last_ready_tick: u64,
}

impl TaskControlBlock {
    pub fn new(
        id: TaskId,
        name: &'static str,
        stack_base: usize,
        stack_size: usize,
        entry_point: usize,
        priority: u8,
        initial_quantum: usize,
        current_tick: u64,
    ) -> Self {
        let mut context = TaskContext::empty();
        let stack_top = stack_base + stack_size;
        unsafe {
            context.init_stack(stack_top, entry_point);
        }

        Self {
            id,
            name,
            state: TaskState::Ready,
            context,
            stack_base,
            stack_size,
            priority,
            assigned_quantum: initial_quantum,
            quantum_remaining: initial_quantum,
            total_cpu_ticks: 0,
            current_burst_ticks: 0,
            burst_history: [initial_quantum; 4],
            burst_count: 0,
            created_at_tick: current_tick,
            context_switches: 0,
            total_wait_ticks: 0,
            last_ready_tick: current_tick,
        }
    }

    /// Records completed CPU burst for dynamic quantum calculation.
    pub fn record_burst(&mut self, burst: usize) {
        let idx = self.burst_count % 4;
        self.burst_history[idx] = burst;
        self.burst_count += 1;
        self.current_burst_ticks = 0;
    }

    /// Computes rounded moving average of recent CPU bursts.
    pub fn recent_average_burst(&self) -> usize {
        if self.burst_count == 0 {
            return self.assigned_quantum;
        }
        let samples = core::cmp::min(self.burst_count, 4);
        let mut sum = 0;
        for i in 0..samples {
            sum += self.burst_history[i];
        }
        (sum + samples / 2) / samples
    }
}
