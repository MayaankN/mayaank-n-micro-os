//! Ready Queue FIFO Implementation
//! Mayaank N - Micro-OS (25BTRCL148)

use crate::task::TaskId;

pub const READY_QUEUE_CAPACITY: usize = 32;

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
            return false; // Queue full
        }
        self.buffer[self.tail] = task_id;
        self.tail = (self.tail + 1) % READY_QUEUE_CAPACITY;
        self.count += 1;
        true
    }

    pub fn pop_front(&mut self) -> Option<TaskId> {
        if self.count == 0 {
            return None; // Queue empty
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

    pub fn remove(&mut self, task_id: TaskId) -> bool {
        let mut found = false;
        let count = self.count;
        let mut new_queue = ReadyQueue::new();

        for _ in 0..count {
            if let Some(id) = self.pop_front() {
                if id == task_id && !found {
                    found = true;
                } else {
                    new_queue.push_back(id);
                }
            }
        }
        *self = new_queue;
        found
    }
}
