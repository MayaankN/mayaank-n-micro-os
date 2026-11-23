//! Ready Queue FIFO Implementation for Verification
//! Mayaank N - Micro-OS (25BTRCL148)

use crate::task::TaskId;

pub const READY_QUEUE_CAPACITY: usize = 32;

#[derive(Debug, Clone)]
pub struct ReadyQueue {
    buffer: [TaskId; READY_QUEUE_CAPACITY],
    head: usize,
    tail: usize,
    count: usize,
}

impl ReadyQueue {
    pub const fn new() -> Self {
        Self {
            buffer: [TaskId(0); READY_QUEUE_CAPACITY],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn push_back(&mut self, task_id: TaskId) -> bool {
        if self.count >= READY_QUEUE_CAPACITY {
            return false;
        }
        self.buffer[self.tail] = task_id;
        self.tail = (self.tail + 1) % READY_QUEUE_CAPACITY;
        self.count += 1;
        true
    }

    pub fn pop_front(&mut self) -> Option<TaskId> {
        if self.count == 0 {
            return None;
        }
        let task_id = self.buffer[self.head];
        self.head = (self.head + 1) % READY_QUEUE_CAPACITY;
        self.count -= 1;
        Some(task_id)
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn contains(&self, task_id: TaskId) -> bool {
        for i in 0..self.count {
            let idx = (self.head + i) % READY_QUEUE_CAPACITY;
            if self.buffer[idx] == task_id {
                return true;
            }
        }
        false
    }
}
