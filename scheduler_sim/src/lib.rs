//! Host-side Scheduler Simulator & Unit Verification Library
//! Mayaank N - Micro-OS (25BTRCL148)

pub mod allocator;
pub mod dynamic_quantum;
pub mod queue;
pub mod scheduler;
pub mod task;

#[cfg(test)]
mod tests;
