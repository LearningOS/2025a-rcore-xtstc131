//! Process management syscalls
use crate::config::PAGE_SIZE;
use crate::task::{change_program_brk, create_new_map_area, current_user_token, exit_current_and_run_next, get_task_syscall_cnt, suspend_current_and_run_next, unmap_consecutive_area};
use crate::timer::get_time_us;
use crate::mm::{translated_byte_buffer, MapPermission, PageTable, VirtAddr};
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
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
    let time_vec = translated_byte_buffer(current_user_token(), _ts as *const u8, core::mem::size_of::<TimeVal>());
    let ref time_val = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
    };
    let src_ptr = time_val as *const TimeVal;
    for (idx, time_v) in time_vec.into_iter().enumerate() {
        let unit_len = time_v.len();
        unsafe {
            time_v.copy_from_slice(core::slice::from_raw_parts(
                src_ptr.wrapping_byte_add(idx * unit_len) as *const u8,
                unit_len)
            );
        }
    }
    0
}

fn _get_value(id: usize) -> isize {
    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    let va = VirtAddr::from(id);

     match page_table.translate(va.floor()) {
            Some(pte) => {
                if pte.readable() {
                    let buffers = translated_byte_buffer(token, id as *const u8, 1);
                    buffers[0][0] as isize
                } else {
                    -1
                }
            }
            None => -1,
     }
}

fn _write_value(id: usize, data: usize) -> isize{
    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    let va = VirtAddr::from(id);

     match page_table.translate(va.floor()) {
            Some(pte) => {
                if pte.writable() {
                    let mut buffers = translated_byte_buffer(token, id as *mut u8, 1);
                    buffers[0][0] = data as u8;
                    0
                } else {
                    -1
                }
            }
            None => -1,
     }
}

fn _get_syscall_num(id: usize) ->isize {
    get_task_syscall_cnt(id) as isize
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    const MAX_VA: usize = (1 << 38) - 1;
    const MIN_VA: usize = (!0) << 38;
    if !(_id <= MAX_VA || _id >= MIN_VA) {
        return -1;
    }
     match _trace_request {
        0 => _get_value(_id),
        1 =>_write_value(_id, _data),
        2 => _get_syscall_num(_id),
        _ => -1
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    if (_start & (PAGE_SIZE - 1) != 0) || (_port & !0x7 != 0) || (_port & 0x7 == 0) {
        return -1;
    }
    let start_vpn = VirtAddr::from(_start).floor();
    let end_vpn = VirtAddr::from(_start + _len).ceil();

    // all ptes in range has pass the test
    let ret = create_new_map_area(
        start_vpn.into(),
        end_vpn.into(),
        MapPermission::from_bits_truncate((_port << 1) as u8) | MapPermission::U
    );
    return ret;
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    if _len == 0 || (_start & (PAGE_SIZE - 1)) != 0 || (_len & (PAGE_SIZE - 1)) != 0 {
        return -1;
    }
        // check the range [start, start + len)
    let start_vpn = VirtAddr::from(_start).floor();
    let end_vpn = VirtAddr::from(_start + _len).ceil();
    unmap_consecutive_area(start_vpn.into(), end_vpn.into())
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
