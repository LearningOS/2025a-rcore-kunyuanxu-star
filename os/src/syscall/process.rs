//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next};

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
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    use crate::mm::translated_byte_buffer;
    use crate::task::current_user_token;
    use crate::timer::get_time_us;

    if ts.is_null() {
        return -1;
    }

    let usec = get_time_us();
    let sec = usec / 1_000_000;
    let usec = usec % 1_000_000;

    // Use translated_byte_buffer to safely access user memory
    let token = current_user_token();
    let ts_buffer = translated_byte_buffer(token, ts as *const u8, core::mem::size_of::<TimeVal>());
    if ts_buffer.is_empty() {
        return -1;
    }

    // Write the time values
    unsafe {
        let ts_ptr = ts as *mut TimeVal;
        (*ts_ptr).sec = sec;
        (*ts_ptr).usec = usec;
    }

    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    use crate::mm::translated_byte_buffer;
    use crate::task::current_user_token;
    use crate::task::TASK_MANAGER;

    match trace_request {
        0 => {
            // Read from user memory
            let token = current_user_token();
            let buffer = translated_byte_buffer(token, id as *const u8, 1);
            if buffer.is_empty() {
                -1
            } else {
                buffer[0][0] as isize
            }
        }
        1 => {
            // Write to user memory
            let token = current_user_token();
            let mut buffer = translated_byte_buffer(token, id as *const u8, 1);
            if buffer.is_empty() {
                -1
            } else {
                buffer[0][0] = data as u8;
                0
            }
        }
        2 => {
            // Get system call count
            TASK_MANAGER.get_syscall_count(id) as isize
        }
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    use crate::task::TASK_MANAGER;
    TASK_MANAGER.mmap_current(start, len, port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    use crate::task::TASK_MANAGER;
    TASK_MANAGER.munmap_current(start, len)
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
