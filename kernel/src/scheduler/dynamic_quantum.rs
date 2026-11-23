//! Dynamic Time Quantum Engine (Scheduler Specialization)
//! Mayaank N - Micro-OS (25BTRCL148)

use crate::task::TaskControlBlock;

pub const MIN_QUANTUM: usize = 2;   // 2 ticks minimum (e.g. 20 ms)
pub const BASE_QUANTUM: usize = 4;  // 4 ticks baseline (e.g. 40 ms)
pub const MAX_QUANTUM: usize = 12;  // 12 ticks maximum (e.g. 120 ms)

pub struct DynamicQuantumEngine {
    min_quantum: usize,
    base_quantum: usize,
    max_quantum: usize,
    quantum_adjustments_count: u64,
}

impl DynamicQuantumEngine {
    pub const fn new() -> Self {
        Self {
            min_quantum: MIN_QUANTUM,
            base_quantum: BASE_QUANTUM,
            max_quantum: MAX_QUANTUM,
            quantum_adjustments_count: 0,
        }
    }

    /// Computes an adapted dynamic time quantum for a task before placing it back in the ready queue.
    ///
    /// Feedback Mechanisms:
    /// 1. CPU Burst History: Tasks exhausting full quantum are rewarded with larger slices (+1 or +2)
    ///    to reduce context switch overhead. Tasks yielding early are scaled down to ensure fast response.
    /// 2. Ready Queue Load: When system queue length is high, quantum is scaled down proportionally
    ///    to bound worst-case wait latency.
    pub fn calculate_quantum(
        &mut self,
        tcb: &TaskControlBlock,
        ready_queue_len: usize,
    ) -> usize {
        self.quantum_adjustments_count += 1;

        let avg_burst = tcb.recent_average_burst();
        let mut quantum = tcb.assigned_quantum;

        // 1. Behavior adaptation based on burst exhaustion
        if avg_burst >= tcb.assigned_quantum {
            // CPU-bound task: expand quantum to boost throughput
            quantum = core::cmp::min(self.max_quantum, quantum + 1);
        } else if avg_burst <= self.min_quantum {
            // Highly interactive / yielding task: shrink quantum for snappy response
            quantum = core::cmp::max(self.min_quantum, quantum.saturating_sub(1));
        } else {
            // Converge towards base quantum
            if quantum > self.base_quantum {
                quantum -= 1;
            } else if quantum < self.base_quantum {
                quantum += 1;
            }
        }

        // 2. Queue Pressure Feedback
        if ready_queue_len > 3 {
            let penalty = core::cmp::min(2, ready_queue_len - 3);
            quantum = core::cmp::max(self.min_quantum, quantum.saturating_sub(penalty));
        }

        // 3. Priority Weighting
        if tcb.priority > 1 {
            quantum = core::cmp::min(self.max_quantum, quantum + (tcb.priority as usize / 2));
        }

        quantum
    }

    pub fn total_adjustments(&self) -> u64 {
        self.quantum_adjustments_count
    }
}
