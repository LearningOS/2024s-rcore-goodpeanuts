//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::config::MAX_SYSCALL_NUM;
use crate::mm::{MapPermission, VirtAddr, VirtPageNum};
use crate::sync::UPSafeCell;
use crate::timer::get_time_ms;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;

/// Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,

    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }

    /// get current task init time
    pub fn get_current_task_init_time(&self) -> usize {
        if self.current.is_none() {
            get_time_ms()
        } else {
            self.current.as_ref().unwrap().inner_exclusive_access().init_time
        }
    }

    /// count syscall times in task control block
    pub fn count_syscall(&mut self, syscall_id: usize) {
        if self.current.is_none() {
            return;
        }
        self.current.as_mut().unwrap().inner_exclusive_access().syscall_cnt[syscall_id] += 1;
    }

    /// get syscall times in task control block
    pub fn get_syscall_times(&self) -> [u32; MAX_SYSCALL_NUM] {
        if self.current.is_none() {
            [0; MAX_SYSCALL_NUM]
        } else {
            self.current.as_ref().unwrap().inner_exclusive_access().syscall_cnt
        }
    }

    /// mmap a area in memory set
    pub fn mmap(&mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission
    ) -> isize {
        if self.current.is_none() {
            return - 1;
        }
        self.current.as_mut().unwrap().inner_exclusive_access().memory_set.insert_framed_area(
            start_va,
            end_va,
            permission
        )        
    }

    /// unmap with strat vpn
    pub fn unmap(&mut self,
        start_vpn: VirtPageNum,
        end_vpn: VirtPageNum,
    ) -> isize {
        if self.current.is_none() {
            return -1;
        }
        self.current.as_mut().unwrap().inner_exclusive_access().memory_set.remove_framed_area(
            start_vpn,
            end_vpn,
        )
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            if task_inner.start == false {
                task_inner.init_time = get_time_ms();
                task_inner.start = true;
            }
            // release coming task_inner manually
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            warn!("no tasks available in run_tasks");
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}

/// get current task run time
pub fn get_current_task_run_time() -> usize {
    get_time_ms() - PROCESSOR.exclusive_access().get_current_task_init_time()
}

/// get syscall count
pub fn get_syscall_times() -> [u32; MAX_SYSCALL_NUM] {
    PROCESSOR.exclusive_access().get_syscall_times()
}

/// count syscall
pub fn count_syscall(syscall_id: usize) {
    PROCESSOR.exclusive_access().count_syscall(syscall_id);
}

/// mmap
pub fn processor_mmap(start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission) -> isize {
    PROCESSOR.exclusive_access().mmap(start_va, end_va, permission)
}

/// mmap
pub fn processor_munmap(start_vpn: VirtPageNum, end_vpn: VirtPageNum) -> isize {
    PROCESSOR.exclusive_access().unmap(start_vpn, end_vpn)
}