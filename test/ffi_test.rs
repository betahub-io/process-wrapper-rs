use ffmpeg_wrapper::{
    ffmpeg_close, ffmpeg_is_running, ffmpeg_read_stderr, ffmpeg_start, ffmpeg_wait,
    ffmpeg_write_stdin,
};
use std::ffi::CString;
use std::thread;
use std::time::Duration;

#[test]
fn test_ffi_process_start() {
    let cmd = CString::new("echo test").unwrap();
    let proc = unsafe { ffmpeg_start(cmd.as_ptr()) };
    assert!(!proc.is_null());
    
    // Clean up
    unsafe { ffmpeg_close(proc) };
}

#[test]
fn test_ffi_null_command() {
    let proc = unsafe { ffmpeg_start(std::ptr::null()) };
    assert!(proc.is_null());
}

#[test]
fn test_ffi_invalid_utf8() {
    let invalid_utf8 = b"echo \xFF test";
    let cmd = unsafe { CString::from_vec_unchecked(invalid_utf8.to_vec()) };
    let proc = unsafe { ffmpeg_start(cmd.as_ptr()) };
    assert!(proc.is_null());
}

#[test]
fn test_ffi_write_stdin() {
    // Start a cat process
    let cmd = CString::new("cat").unwrap();
    let proc = unsafe { ffmpeg_start(cmd.as_ptr()) };
    assert!(!proc.is_null());
    
    // Write to stdin
    let data = b"test data";
    let result = unsafe { ffmpeg_write_stdin(proc, data.as_ptr(), data.len()) };
    assert!(result > 0);
    
    // Clean up
    unsafe { ffmpeg_close(proc) };
}

#[test]
fn test_ffi_stderr_capture() {
    // Start a process that outputs to stderr
    let cmd = CString::new("sh -c 'echo error message >&2'").unwrap();
    let proc = unsafe { ffmpeg_start(cmd.as_ptr()) };
    assert!(!proc.is_null());
    
    // Wait for stderr to be captured
    thread::sleep(Duration::from_millis(100));
    
    // Read from stderr
    let mut buf = [0u8; 1024];
    let result = unsafe { ffmpeg_read_stderr(proc, buf.as_mut_ptr(), buf.len()) };
    assert!(result > 0);
    
    // Verify the stderr output
    let stderr_output = std::str::from_utf8(&buf[..result as usize]).unwrap();
    assert!(stderr_output.contains("error message"));
    
    // Wait for the process
    let exit_code = unsafe { ffmpeg_wait(proc) };
    assert_eq!(exit_code, 0);
    
    // Clean up
    unsafe { ffmpeg_close(proc) };
}

#[test]
fn test_ffi_is_running() {
    // Start a process that sleeps for a short time
    let cmd = CString::new("sleep 0.1").unwrap();
    let proc = unsafe { ffmpeg_start(cmd.as_ptr()) };
    assert!(!proc.is_null());
    
    // Check if it's running
    let is_running = unsafe { ffmpeg_is_running(proc) };
    assert_eq!(is_running, 1);
    
    // Wait for it to exit
    thread::sleep(Duration::from_millis(200));
    
    // Check if it's still running (it shouldn't be)
    let is_running = unsafe { ffmpeg_is_running(proc) };
    assert_eq!(is_running, 0);
    
    // Wait for it
    let exit_code = unsafe { ffmpeg_wait(proc) };
    assert_eq!(exit_code, 0);
    
    // Clean up
    unsafe { ffmpeg_close(proc) };
}

#[test]
fn test_ffi_wait() {
    // Start a process that exits with a specific code
    let cmd = CString::new("sh -c 'exit 42'").unwrap();
    let proc = unsafe { ffmpeg_start(cmd.as_ptr()) };
    assert!(!proc.is_null());
    
    // Wait for it
    let exit_code = unsafe { ffmpeg_wait(proc) };
    assert_eq!(exit_code, 42);
    
    // Clean up
    unsafe { ffmpeg_close(proc) };
}

#[test]
fn test_ffi_null_process() {
    // Test with null process pointers
    let result = unsafe { ffmpeg_write_stdin(std::ptr::null_mut(), b"test".as_ptr(), 4) };
    assert_eq!(result, -1);
    
    let mut buf = [0u8; 10];
    let result = unsafe { ffmpeg_read_stderr(std::ptr::null_mut(), buf.as_mut_ptr(), 10) };
    assert_eq!(result, -1);
    
    let is_running = unsafe { ffmpeg_is_running(std::ptr::null_mut()) };
    assert_eq!(is_running, 0);
    
    let exit_code = unsafe { ffmpeg_wait(std::ptr::null_mut()) };
    assert_eq!(exit_code, -1);
    
    // This should not crash
    unsafe { ffmpeg_close(std::ptr::null_mut()) };
} 