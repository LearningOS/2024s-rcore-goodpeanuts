//! Process management syscalls
use core::mem::size_of;
use crate::mm::{MapPermission, VirtAddr};

use crate::task::{process_mmap, process_munmap};
use crate::{
    config::MAX_SYSCALL_NUM, mm::translated_byte_buffer, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_task_run_time, get_syscall_times, suspend_current_and_run_next, TaskStatus
    }, timer::get_time_us
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();

    let mut buffer = translated_byte_buffer(current_user_token(), _ts as *const u8, size_of::<TimeVal>());
    // Write the seconds and microseconds to the buffer.
    let time_val_ptr = buffer[0].as_mut_ptr() as *mut TimeVal;
    unsafe {
        (*time_val_ptr).sec = us / 1_000_000;
        (*time_val_ptr).usec = us % 1_000_000;
    }

    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let mut buffer = translated_byte_buffer(current_user_token(), _ti as *const u8, size_of::<TaskInfo>());
    let task_info_ptr = buffer[0].as_mut_ptr() as *mut TaskInfo;
    unsafe {
        (*task_info_ptr).status = TaskStatus::Running;
        (*task_info_ptr).syscall_times = get_syscall_times();
        (*task_info_ptr).time = get_current_task_run_time();
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    let virt_start = VirtAddr::from(_start);
    if virt_start.aligned() == false || _port & !0x7 != 0 || _port& 0x7 == 0 {
        return -1;
    }
    let virt_end = VirtAddr::from((_start + _len + 4095) / 4096 * 4096);
    // let permission = MapPermission::from_bits(_port as u8).unwrap() | MapPermission::U; 
    // KERNEL_SPACE.exclusive_access().insert_framed_area(virt_sart, virt_end, permission);

    let mut map_p = MapPermission::U;
    if (_port & 0b0001) != 0 {
        map_p = map_p | MapPermission::R;
    }
    if (_port & 0b0010) != 0 {
        map_p = map_p | MapPermission::W;
    }
    if (_port & 0b0100) != 0 {
        map_p = map_p | MapPermission::X;
    }


    process_mmap(virt_start, virt_end, map_p)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let virt_start = VirtAddr::from(_start);
    if virt_start.aligned() == false {
        return -1;
    }
    let virt_end = VirtAddr::from(_start + _len);
    process_munmap(virt_start, virt_end)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
