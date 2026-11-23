//! CPU Scheduling Subsystem
//! Mayaank N - Micro-OS (25BTRCL148)

pub mod dynamic_quantum;
pub mod queue;
pub mod round_robin;

pub use dynamic_quantum::{DynamicQuantumEngine, BASE_QUANTUM, MAX_QUANTUM, MIN_QUANTUM};
pub use queue::{ReadyQueue, READY_QUEUE_CAPACITY};
pub use round_robin::{
    switch_context, yield_now, RoundRobinScheduler, MAX_TASKS, SCHEDULER,
};
