//! Round Robin Scheduler with Dynamic Quantum Model for Simulation & Metrics
//! Mayaank N - Micro-OS (25BTRCL148)

use crate::dynamic_quantum::{DynamicQuantumEngine, BASE_QUANTUM};
use crate::queue::ReadyQueue;
use crate::task::{TaskControlBlock, TaskId, TaskState};

pub const MAX_TASKS: usize = 16;

#[derive(Debug)]
pub struct RoundRobinScheduler {
    pub tasks: [Option<TaskControlBlock>; MAX_TASKS],
    pub ready_queue: ReadyQueue,
    pub current_task: Option<TaskId>,
    pub dynamic_quantum_engine: DynamicQuantumEngine,
    pub total_context_switches: u64,
    pub current_tick: u64,
}

impl RoundRobinScheduler {
    pub const fn new() -> Self {
        const NONE_TCB: Option<TaskControlBlock> = None;
        Self {
            tasks: [NONE_TCB; MAX_TASKS],
            ready_queue: ReadyQueue::new(),
            current_task: None,
            dynamic_quantum_engine: DynamicQuantumEngine::new(),
            total_context_switches: 0,
            current_tick: 0,
        }
    }

    pub fn spawn_task(&mut self, id: TaskId, name: &str, priority: u8) -> bool {
        let slot = id.0 as usize;
        if slot >= MAX_TASKS || self.tasks[slot].is_some() {
            return false;
        }

        let tcb = TaskControlBlock::new(id, name, priority, BASE_QUANTUM, self.current_tick);
        self.tasks[slot] = Some(tcb);
        self.ready_queue.push_back(id);
        true
    }

    pub fn schedule_next(&mut self) -> Option<(TaskId, usize)> {
        // 1. Handle previous running task
        if let Some(curr_id) = self.current_task {
            let slot = curr_id.0 as usize;
            if let Some(ref mut tcb) = self.tasks[slot] {
                if tcb.state == TaskState::Running {
                    let burst = tcb.current_burst_ticks;
                    tcb.record_burst(burst);

                    let new_quantum = self.dynamic_quantum_engine.calculate_quantum(
                        tcb,
                        self.ready_queue.len(),
                    );
                    tcb.assigned_quantum = new_quantum;
                    tcb.quantum_remaining = new_quantum;
                    tcb.state = TaskState::Ready;
                    tcb.last_ready_tick = self.current_tick;

                    self.ready_queue.push_back(curr_id);
                }
            }
        }

        // 2. Select next task
        let next_id = match self.ready_queue.pop_front() {
            Some(id) => id,
            None => return None,
        };

        let next_slot = next_id.0 as usize;
        let quantum = match self.tasks[next_slot] {
            Some(ref mut tcb) => {
                tcb.state = TaskState::Running;
                tcb.context_switches += 1;
                let wait = self.current_tick.saturating_sub(tcb.last_ready_tick);
                tcb.total_wait_ticks += wait;
                tcb.assigned_quantum
            }
            None => return None,
        };

        self.current_task = Some(next_id);
        self.total_context_switches += 1;

        Some((next_id, quantum))
    }

    pub fn step_tick(&mut self) -> bool {
        self.current_tick += 1;
        if let Some(curr_id) = self.current_task {
            let slot = curr_id.0 as usize;
            if let Some(ref mut tcb) = self.tasks[slot] {
                tcb.total_cpu_ticks += 1;
                tcb.current_burst_ticks += 1;
                if tcb.quantum_remaining > 0 {
                    tcb.quantum_remaining -= 1;
                }
                if tcb.quantum_remaining == 0 {
                    return true; // Quantum expired
                }
            }
        }
        false
    }
}
