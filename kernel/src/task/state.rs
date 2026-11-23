//! Task Lifecycle State Machine
//! Mayaank N - Micro-OS (25BTRCL148)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

impl TaskState {
    pub fn is_ready(&self) -> bool {
        matches!(self, TaskState::Ready)
    }

    pub fn is_running(&self) -> bool {
        matches!(self, TaskState::Running)
    }

    pub fn is_terminated(&self) -> bool {
        matches!(self, TaskState::Terminated)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TaskState::Ready => "READY",
            TaskState::Running => "RUNNING",
            TaskState::Blocked => "BLOCKED",
            TaskState::Terminated => "TERMINATED",
        }
    }
}
