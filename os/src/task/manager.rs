//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready: Vec<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready: Vec::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready.push(task);
    }
    /// Fetch the runnable task with the smallest stride (linear O(n))
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        if self.ready.is_empty() {
            return None;
        }
        // find index of min stride
        let mut min_idx = 0usize;
        let mut min_stride = {
            let inner = self.ready[0].inner_exclusive_access();
            inner.stride
        };
        for i in 1..self.ready.len() {
            let s = {
                let inner = self.ready[i].inner_exclusive_access();
                inner.stride
            };
            if s < min_stride {
                min_stride = s;
                min_idx = i;
            }
        }
        // O(1) remove (unordered)
        Some(self.ready.swap_remove(min_idx))
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}

/// When a task has used up its time slice and is still runnable:
/// increase its stride by its pass value, then put it back into the ready queue.
pub fn requeue_after_timeslice(task: Arc<TaskControlBlock>) {
    {
        let mut inner = task.inner_exclusive_access();
        // Update stride to record that this task has consumed one time slice
        inner.stride = inner.stride.saturating_add(inner.pass);
    }
    // Reinsert the task into the ready queue
    add_task(task);
}