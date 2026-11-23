//! Task Management Models for Verification & Simulation
//! Mayaank N - Micro-OS (25BTRCL148)

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

#[derive(Debug, Clone)]
pub struct TaskControlBlock {
    pub id: TaskId,
    pub name: String,
    pub state: TaskState,
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
        name: &str,
        priority: u8,
        initial_quantum: usize,
        current_tick: u64,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            state: TaskState::Ready,
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

    pub fn record_burst(&mut self, burst: usize) {
        let idx = self.burst_count % 4;
        self.burst_history[idx] = burst;
        self.burst_count += 1;
        self.current_burst_ticks = 0;
    }

    pub fn recent_average_burst(&self) -> usize {
        if self.burst_count == 0 {
            return self.assigned_quantum;
        }
        let samples = std::cmp::min(self.burst_count, 4);
        let mut sum = 0;
        for i in 0..samples {
            sum += self.burst_history[i];
        }
        (sum + samples / 2) / samples
    }
}
