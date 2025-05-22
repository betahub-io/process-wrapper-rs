use crate::process::Process;
use libc::{c_char, c_int, size_t};
use std::ffi::CStr;
use std::ptr;

/// Start a new process with the given command line
///
/// # Safety
///
/// `cmd` must be a valid null-terminated C string.
#[no_mangle]
pub extern "C" fn process_start(cmd: *const c_char) -> *mut Process {
    // Safety check
    if cmd.is_null() {
        return ptr::null_mut();
    }
    
    // Convert C string to Rust string
    let cmd_str = unsafe {
        match CStr::from_ptr(cmd).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    // Create the process
    match Process::new(cmd_str) {
        Ok(process) => Box::into_raw(Box::new(process)),
        Err(_) => ptr::null_mut(),
    }
}

/// Start a new process with the given program path and arguments
///
/// # Safety
///
/// `program` must be a valid null-terminated C string.
/// `args` must be an array of valid null-terminated C strings.
/// `args_len` must be the length of the `args` array.
/// The last element of `args` must be a null pointer.
#[no_mangle]
pub extern "C" fn process_start_with_args(
    program: *const c_char,
    args: *const *const c_char,
    args_len: size_t,
) -> *mut Process {
    // Safety check
    if program.is_null() || (args.is_null() && args_len > 0) {
        return ptr::null_mut();
    }
    
    // Convert program C string to Rust string
    let program_str = unsafe {
        match CStr::from_ptr(program).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    // Convert args C strings to Rust strings
    let mut args_vec = Vec::with_capacity(args_len);
    
    unsafe {
        for i in 0..args_len {
            let arg_ptr = *args.add(i);
            if arg_ptr.is_null() {
                break;
            }
            
            match CStr::from_ptr(arg_ptr).to_str() {
                Ok(s) => args_vec.push(s),
                Err(_) => return ptr::null_mut(),
            }
        }
    }
    
    // Create the process
    match Process::new_with_args(program_str, &args_vec) {
        Ok(process) => Box::into_raw(Box::new(process)),
        Err(_) => ptr::null_mut(),
    }
}

/// Write data to the process's stdin
///
/// # Safety
///
/// `proc` must be a valid pointer returned by `process_start`.
/// `data` must be a valid pointer to a buffer of at least `len` bytes.
#[no_mangle]
pub extern "C" fn process_write_stdin(
    proc: *mut Process,
    data: *const u8,
    len: size_t,
) -> isize {
    // Safety checks
    if proc.is_null() || data.is_null() || len == 0 {
        return -1;
    }
    
    // Get the process
    let process = unsafe { &mut *proc };
    
    // Convert the data
    let data_slice = unsafe { std::slice::from_raw_parts(data, len) };
    
    // Write to stdin
    match process.write_stdin(data_slice) {
        Ok(bytes_written) => bytes_written as isize,
        Err(_) => -1,
    }
}

/// Read data from the process's stderr
///
/// # Safety
///
/// `proc` must be a valid pointer returned by `process_start`.
/// `buf` must be a valid pointer to a buffer of at least `len` bytes.
#[no_mangle]
pub extern "C" fn process_read_stderr(
    proc: *mut Process,
    buf: *mut u8,
    len: size_t,
) -> isize {
    // Safety checks
    if proc.is_null() || buf.is_null() || len == 0 {
        return -1;
    }
    
    // Get the process
    let process = unsafe { &mut *proc };
    
    // Create a mutable slice for the buffer
    let buf_slice = unsafe { std::slice::from_raw_parts_mut(buf, len) };
    
    // Read from stderr
    match process.read_stderr(buf_slice) {
        Ok(bytes_read) => bytes_read as isize,
        Err(_) => -1,
    }
}

/// Check if the process is still running
///
/// # Safety
///
/// `proc` must be a valid pointer returned by `process_start`.
#[no_mangle]
pub extern "C" fn process_is_running(proc: *mut Process) -> c_int {
    // Safety check
    if proc.is_null() {
        return 0;
    }
    
    // Get the process
    let process = unsafe { &mut *proc };
    
    // Check if running
    if process.is_running() {
        1
    } else {
        0
    }
}

/// Wait for the process to exit and return the exit code
///
/// # Safety
///
/// `proc` must be a valid pointer returned by `process_start`.
#[no_mangle]
pub extern "C" fn process_wait(proc: *mut Process) -> c_int {
    // Safety check
    if proc.is_null() {
        return -1;
    }
    
    // Get the process
    let process = unsafe { &mut *proc };
    
    // Wait for the process
    match process.wait() {
        Ok(exit_code) => exit_code,
        Err(_) => -1,
    }
}

/// Close stdin, terminate the process, and clean up resources
///
/// # Safety
///
/// `proc` must be a valid pointer returned by `process_start`.
#[no_mangle]
pub extern "C" fn process_close(proc: *mut Process) {
    // Safety check
    if proc.is_null() {
        return;
    }
    
    // Get the process
    let process = unsafe { &mut *proc };
    
    // Close the process
    let _ = process.close();
} 