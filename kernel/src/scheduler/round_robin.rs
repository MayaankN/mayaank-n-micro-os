//! Round Robin Scheduler with Dynamic Quantum Adaptation
//! Mayaank N - Micro-OS (25BTRCL148)

use crate::klog_sched;
use crate::scheduler::dynamic_quantum::{DynamicQuantumEngine, BASE_QUANTUM};
use crate::scheduler::queue::ReadyQueue;
use crate::sync::SpinLock;
use crate::task::{TaskControlBlock, TaskContext, TaskId, TaskState, DEFAULT_STACK_SIZE};
use core::arch::naked_asm;

pub const MAX_TASKS: usize = 16;

pub struct RoundRobinScheduler {
    tasks: [Option<TaskControlBlock>; MAX_TASKS],
    ready_queue: ReadyQueue,
    current_task: Option<TaskId>,
    pub idle_task_id: TaskId,
    dynamic_quantum_engine: DynamicQuantumEngine,
    total_context_switches: u64,
    scheduler_ticks: u64,
}

impl RoundRobinScheduler {
    pub const fn new() -> Self {
        const NONE_TCB: Option<TaskControlBlock> = None;
        Self {
            tasks: [NONE_TCB; MAX_TASKS],
            ready_queue: ReadyQueue::new(),
            current_task: None,
            idle_task_id: TaskId(0),
            dynamic_quantum_engine: DynamicQuantumEngine::new(),
            total_context_switches: 0,
            scheduler_ticks: 0,
        }
    }

    /// Spawns a new task in the scheduler.
    pub fn spawn_task(
        &mut self,
        id: TaskId,
        name: &'static str,
        entry_point: usize,
        priority: u8,
        stack_base: usize,
        current_tick: u64,
    ) -> bool {
        let slot = id.0 as usize;
        if slot >= MAX_TASKS || self.tasks[slot].is_some() {
            return false;
        }

        let tcb = TaskControlBlock::new(
            id,
            name,
            stack_base,
            DEFAULT_STACK_SIZE,
            entry_point,
            priority,
            BASE_QUANTUM,
            current_tick,
        );

        self.tasks[slot] = Some(tcb);
        self.ready_queue.push_back(id);
        true
    }

    /// Timer tick invocation from PIT interrupt handler.
    pub fn on_timer_tick(&mut self, _current_tick: u64) -> bool {
        self.scheduler_ticks += 1;

        if let Some(curr_id) = self.current_task {
            let slot = curr_id.0 as usize;
            if let Some(ref mut tcb) = self.tasks[slot] {
                tcb.total_cpu_ticks += 1;
                tcb.current_burst_ticks += 1;

                if tcb.quantum_remaining > 0 {
                    tcb.quantum_remaining -= 1;
                }

                // If quantum expired, request reschedule
                if tcb.quantum_remaining == 0 {
                    return true;
                }
            }
        } else {
            return true;
        }

        false
    }

    /// Selects the next ready task and returns (old_context, new_context) for switching.
    pub fn schedule_next(
        &mut self,
        current_tick: u64,
    ) -> Option<(*mut TaskContext, *const TaskContext, TaskId, TaskId, usize)> {
        let mut old_context_ptr: *mut TaskContext = core::ptr::null_mut();
        let mut prev_id = TaskId(0);

        // 1. Handle currently running task
        if let Some(curr_id) = self.current_task {
            prev_id = curr_id;
            let slot = curr_id.0 as usize;
            if let Some(ref mut tcb) = self.tasks[slot] {
                if tcb.state == TaskState::Running {
                    // Record burst and adapt dynamic quantum
                    let burst = tcb.current_burst_ticks;
                    tcb.record_burst(burst);

                    let new_quantum = self.dynamic_quantum_engine.calculate_quantum(
                        tcb,
                        self.ready_queue.len(),
                    );
                    tcb.assigned_quantum = new_quantum;
                    tcb.quantum_remaining = new_quantum;
                    tcb.state = TaskState::Ready;
                    tcb.last_ready_tick = current_tick;

                    // Re-enqueue into ready queue
                    self.ready_queue.push_back(curr_id);
                }
                old_context_ptr = &mut tcb.context as *mut TaskContext;
            }
        }

        // 2. Pop next ready task
        let next_id = match self.ready_queue.pop_front() {
            Some(id) => id,
            None => {
                // If ready queue is empty, keep running current task or idle
                if let Some(curr_id) = self.current_task {
                    curr_id
                } else {
                    return None;
                }
            }
        };

        let next_slot = next_id.0 as usize;
        let (new_context_ptr, assigned_q) = match self.tasks[next_slot] {
            Some(ref mut next_tcb) => {
                next_tcb.state = TaskState::Running;
                next_tcb.context_switches += 1;
                let wait_time = current_tick.saturating_sub(next_tcb.last_ready_tick);
                next_tcb.total_wait_ticks += wait_time;
                let q = next_tcb.assigned_quantum;
                (&next_tcb.context as *const TaskContext, q)
            }
            None => return None,
        };

        self.current_task = Some(next_id);
        self.total_context_switches += 1;

        Some((old_context_ptr, new_context_ptr, prev_id, next_id, assigned_q))
    }

    pub fn current_task_id(&self) -> Option<TaskId> {
        self.current_task
    }

    pub fn total_context_switches(&self) -> u64 {
        self.total_context_switches
    }

    pub fn ready_queue_len(&self) -> usize {
        self.ready_queue.len()
    }

    pub fn get_task_info(&self, id: TaskId) -> Option<(&'static str, TaskState, usize, u64, u64)> {
        let slot = id.0 as usize;
        if slot < MAX_TASKS {
            if let Some(ref tcb) = self.tasks[slot] {
                return Some((
                    tcb.name,
                    tcb.state,
                    tcb.assigned_quantum,
                    tcb.total_cpu_ticks,
                    tcb.context_switches,
                ));
            }
        }
        None
    }
}

pub static SCHEDULER: SpinLock<RoundRobinScheduler> = SpinLock::new(RoundRobinScheduler::new());

/// Cooperative yield function callable from any kernel task.
pub fn yield_now() {
    let current_tick = crate::timer::current_ticks();
    let switch_data = {
        let mut sched = SCHEDULER.lock_irq_disabled();
        sched.schedule_next(current_tick)
    };

    if let Some((old_ctx, new_ctx, prev_id, next_id, quantum)) = switch_data {
        if prev_id != next_id && !old_ctx.is_null() && !new_ctx.is_null() {
            klog_sched!(
                "Yield Switch: {} -> {} | Dynamic Quantum: {} ticks",
                prev_id,
                next_id,
                quantum
            );
            unsafe {
                switch_context(old_ctx, new_ctx);
            }
        }
    }
}

/// Low-level x86_64 context switch routine.
/// RDI: *mut TaskContext (old context, points to old.rsp)
/// RSI: *const TaskContext (new context, points to new.rsp)
#[unsafe(naked)]
pub unsafe extern "C" fn switch_context(old: *mut TaskContext, new: *const TaskContext) {
    naked_asm!(
        // Save current task's callee-saved registers onto its stack
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Save current stack pointer to old context struct: [rdi] = rsp
        "mov [rdi], rsp",

        // Load next task's stack pointer from new context struct: rsp = [rsi]
        "mov rsp, [rsi]",

        // Restore next task's callee-saved registers from its stack
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",

        // Return into the next task's saved RIP
        "ret"
    );
}
