//! Dynamic Quantum Engine Model for Verification
//! Mayaank N - Micro-OS (25BTRCL148)

use crate::task::TaskControlBlock;

pub const MIN_QUANTUM: usize = 2;
pub const BASE_QUANTUM: usize = 4;
pub const MAX_QUANTUM: usize = 12;

#[derive(Debug, Clone)]
pub struct DynamicQuantumEngine {
    pub min_quantum: usize,
    pub base_quantum: usize,
    pub max_quantum: usize,
    pub adjustments_count: u64,
}

impl DynamicQuantumEngine {
    pub const fn new() -> Self {
        Self {
            min_quantum: MIN_QUANTUM,
            base_quantum: BASE_QUANTUM,
            max_quantum: MAX_QUANTUM,
            adjustments_count: 0,
        }
    }

    pub fn calculate_quantum(&mut self, tcb: &TaskControlBlock, ready_queue_len: usize) -> usize {
        self.adjustments_count += 1;
        let avg_burst = tcb.recent_average_burst();
        let mut quantum = tcb.assigned_quantum;

        // 1. Burst behavior adaptation
        if avg_burst >= tcb.assigned_quantum {
            quantum = std::cmp::min(self.max_quantum, quantum + 1);
        } else if avg_burst <= self.min_quantum {
            quantum = std::cmp::max(self.min_quantum, quantum.saturating_sub(1));
        } else {
            if quantum > self.base_quantum {
                quantum -= 1;
            } else if quantum < self.base_quantum {
                quantum += 1;
            }
        }

        // 2. Queue pressure feedback
        if ready_queue_len > 3 {
            let penalty = std::cmp::min(2, ready_queue_len - 3);
            quantum = std::cmp::max(self.min_quantum, quantum.saturating_sub(penalty));
        }

        // 3. Priority weighting
        if tcb.priority > 1 {
            quantum = std::cmp::min(self.max_quantum, quantum + (tcb.priority as usize / 2));
        }

        quantum
    }
}
