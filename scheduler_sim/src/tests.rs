//! Automated Test Suite for Micro-OS Verification
//! Student: Mayaank N (25BTRCL148)

use crate::allocator::BumpAllocator;
use crate::dynamic_quantum::{DynamicQuantumEngine, BASE_QUANTUM, MAX_QUANTUM, MIN_QUANTUM};
use crate::queue::{ReadyQueue, READY_QUEUE_CAPACITY};
use crate::scheduler::RoundRobinScheduler;
use crate::task::{TaskControlBlock, TaskId, TaskState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

#[test]
fn test_task_creation() {
    let tcb = TaskControlBlock::new(TaskId(1), "Test-Worker", 1, BASE_QUANTUM, 0);
    assert_eq!(tcb.id, TaskId(1));
    assert_eq!(tcb.name, "Test-Worker");
    assert_eq!(tcb.state, TaskState::Ready);
    assert_eq!(tcb.priority, 1);
    assert_eq!(tcb.assigned_quantum, BASE_QUANTUM);
    assert_eq!(tcb.quantum_remaining, BASE_QUANTUM);
    assert_eq!(tcb.total_cpu_ticks, 0);
    assert_eq!(tcb.context_switches, 0);
}

#[test]
fn test_task_state_transitions() {
    let mut tcb = TaskControlBlock::new(TaskId(2), "State-Task", 1, 4, 0);
    assert_eq!(tcb.state, TaskState::Ready);

    tcb.state = TaskState::Running;
    assert_eq!(tcb.state, TaskState::Running);

    tcb.state = TaskState::Blocked;
    assert_eq!(tcb.state, TaskState::Blocked);

    tcb.state = TaskState::Terminated;
    assert_eq!(tcb.state, TaskState::Terminated);
}

#[test]
fn test_ready_queue_ordering() {
    let mut rq = ReadyQueue::new();
    assert!(rq.is_empty());

    assert!(rq.push_back(TaskId(10)));
    assert!(rq.push_back(TaskId(20)));
    assert!(rq.push_back(TaskId(30)));
    assert_eq!(rq.len(), 3);

    // FIFO order verification
    assert_eq!(rq.pop_front(), Some(TaskId(10)));
    assert_eq!(rq.pop_front(), Some(TaskId(20)));
    assert_eq!(rq.pop_front(), Some(TaskId(30)));
    assert_eq!(rq.pop_front(), None);
    assert!(rq.is_empty());
}

#[test]
fn test_ready_queue_boundary_capacity() {
    let mut rq = ReadyQueue::new();
    for i in 0..READY_QUEUE_CAPACITY {
        assert!(rq.push_back(TaskId(i as u64)));
    }
    assert_eq!(rq.len(), READY_QUEUE_CAPACITY);

    // Boundary: Pushing beyond capacity must fail safely without crash or overwrite
    assert!(!rq.push_back(TaskId(999)));
    assert_eq!(rq.len(), READY_QUEUE_CAPACITY);
}

#[test]
fn test_round_robin_scheduling() {
    let mut sched = RoundRobinScheduler::new();
    assert!(sched.spawn_task(TaskId(1), "Worker-1", 1));
    assert!(sched.spawn_task(TaskId(2), "Worker-2", 1));
    assert!(sched.spawn_task(TaskId(3), "Worker-3", 1));

    let first = sched.schedule_next().unwrap();
    assert_eq!(first.0, TaskId(1));

    let second = sched.schedule_next().unwrap();
    assert_eq!(second.0, TaskId(2));

    let third = sched.schedule_next().unwrap();
    assert_eq!(third.0, TaskId(3));

    // Cycle repeats back to first task
    let fourth = sched.schedule_next().unwrap();
    assert_eq!(fourth.0, TaskId(1));
    assert_eq!(sched.total_context_switches, 4);
}

#[test]
fn test_dynamic_quantum_cpu_bound_expansion() {
    let mut engine = DynamicQuantumEngine::new();
    let mut tcb = TaskControlBlock::new(TaskId(1), "CPU-Bound", 1, BASE_QUANTUM, 0);

    // Initial burst exhaust: 4 >= 4 expands quantum
    tcb.record_burst(4);
    let q1 = engine.calculate_quantum(&tcb, 1);
    assert!(q1 > BASE_QUANTUM, "CPU-bound task must receive expanded quantum");

    // Sustained high bursts
    tcb.assigned_quantum = q1;
    tcb.record_burst(q1);
    tcb.record_burst(q1);
    tcb.record_burst(q1);
    let q2 = engine.calculate_quantum(&tcb, 1);
    assert!(q2 >= q1, "Sustained CPU-bound workload must maintain or expand quantum");
    assert!(q2 <= MAX_QUANTUM);
}

#[test]
fn test_dynamic_quantum_interactive_reduction() {
    let mut engine = DynamicQuantumEngine::new();
    let mut tcb = TaskControlBlock::new(TaskId(2), "Interactive", 1, BASE_QUANTUM, 0);

    // Simulate task yielding early with burst of 1 tick
    tcb.record_burst(1);
    let q1 = engine.calculate_quantum(&tcb, 1);
    assert!(q1 < BASE_QUANTUM, "Interactive task must receive smaller quantum for lower latency");
    assert!(q1 >= MIN_QUANTUM);
}

#[test]
fn test_dynamic_quantum_queue_pressure() {
    let mut engine = DynamicQuantumEngine::new();
    let mut tcb = TaskControlBlock::new(TaskId(3), "Normal", 1, BASE_QUANTUM, 0);
    tcb.record_burst(3);

    // Under heavy queue pressure (depth = 6 > 3)
    let q_loaded = engine.calculate_quantum(&tcb, 6);
    let q_idle = engine.calculate_quantum(&tcb, 1);
    assert!(q_loaded <= q_idle, "Heavy queue load must decrease quantum to protect latency");
}

#[test]
fn test_timer_ticks_and_quantum_expiration() {
    let mut sched = RoundRobinScheduler::new();
    assert!(sched.spawn_task(TaskId(1), "Timer-Task", 1));
    let (_, q) = sched.schedule_next().unwrap();
    assert_eq!(q, BASE_QUANTUM);

    // Step ticks up to quantum - 1
    for _ in 0..(BASE_QUANTUM - 1) {
        let expired = sched.step_tick();
        assert!(!expired);
    }

    // Next tick must expire quantum
    let expired = sched.step_tick();
    assert!(expired, "Quantum must expire after exact assigned ticks");
}

#[test]
fn test_synchronization_mutual_exclusion() {
    let lock = Arc::new(AtomicBool::new(false));
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..8 {
        let l = Arc::clone(&lock);
        let c = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                // Spinlock acquire
                while l.swap(true, Ordering::Acquire) {
                    std::hint::spin_loop();
                }
                c.fetch_add(1, Ordering::Relaxed);
                // Spinlock release
                l.store(false, Ordering::Release);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(counter.load(Ordering::SeqCst), 8000);
}

#[test]
fn test_bump_allocator_allocation_and_alignment() {
    let mut alloc = BumpAllocator::new();
    let heap_base = 0x100000;
    let heap_size = 64 * 1024;
    alloc.init(heap_base, heap_size);

    let p1 = alloc.allocate(100, 16).unwrap();
    assert_eq!(p1 % 16, 0, "Address must be 16-byte aligned");

    let p2 = alloc.allocate(50, 32).unwrap();
    assert_eq!(p2 % 32, 0, "Address must be 32-byte aligned");
    assert!(p2 >= p1 + 100);

    assert_eq!(alloc.allocation_count(), 2);
    assert_eq!(alloc.total_capacity(), heap_size);
    assert!(alloc.used_bytes() >= 150);
}

#[test]
fn test_bump_allocator_out_of_memory_boundary() {
    let mut alloc = BumpAllocator::new();
    alloc.init(0x10000, 1024);

    // Valid allocation
    assert!(alloc.allocate(512, 8).is_some());

    // Exceeding remaining memory must return None
    assert!(alloc.allocate(1000, 8).is_none());
    assert_eq!(alloc.allocation_count(), 1);
}

#[test]
fn test_invalid_operations() {
    let mut sched = RoundRobinScheduler::new();
    assert!(sched.spawn_task(TaskId(1), "Task-1", 1));

    // Duplicate TaskId spawn must fail
    assert!(!sched.spawn_task(TaskId(1), "Duplicate-Task", 1));

    // Out-of-bounds TaskId spawn must fail
    assert!(!sched.spawn_task(TaskId(99), "Invalid-Task", 1));

    // Empty queue pop must safely return None
    let mut empty_rq = ReadyQueue::new();
    assert_eq!(empty_rq.pop_front(), None);
}
