use betahub_process_wrapper::*;
use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_process_version() {
    // Check if a command can run successfully
    let cmd = CString::new("echo version").unwrap();
    let proc = unsafe { process_start(cmd.as_ptr()) };
    assert!(!proc.is_null());
    
    // Wait for the process to exit
    let exit_code = unsafe { process_wait(proc) };
    assert_eq!(exit_code, 0);
    
    // Clean up
    unsafe { process_close(proc) };
}

#[test]
fn test_process_error_output() {
    // Run a command with an invalid option
    let cmd = CString::new("ls -invalid-option").unwrap();
    let proc = unsafe { process_start(cmd.as_ptr()) };
    assert!(!proc.is_null());
    
    // Wait for stderr to be captured
    thread::sleep(Duration::from_millis(300));
    
    // Read from stderr
    let mut buf = [0u8; 1024];
    let result = unsafe { process_read_stderr(proc, buf.as_mut_ptr(), buf.len()) };
    assert!(result > 0);
    
    // Verify that the stderr output contains an error message
    let stderr_output = std::str::from_utf8(&buf[..result as usize]).unwrap();
    println!("Stderr output: {}", stderr_output);
    assert!(stderr_output.contains("unrecognized") || 
            stderr_output.contains("invalid") || 
            stderr_output.contains("illegal") || 
            stderr_output.contains("unknown") ||
            stderr_output.contains("usage"));
    
    // The process should exit with a non-zero code
    let exit_code = unsafe { process_wait(proc) };
    assert_ne!(exit_code, 0);
    
    // Clean up
    unsafe { process_close(proc) };
}

#[test]
#[ignore] // This test requires cat to be installed
fn test_file_processing() {
    // Create a temporary directory for our test files
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("input.txt");
    let output_path = dir.path().join("output.txt");
    
    // Create a test input file
    create_test_file(&input_path, "Hello, world!\nThis is a test file.\n");
    
    // Construct the command
    let cmd = CString::new(format!(
        "cat > {}",
        output_path.to_str().unwrap()
    ))
    .unwrap();
    
    // Start the process
    let proc = unsafe { process_start(cmd.as_ptr()) };
    assert!(!proc.is_null());
    
    // Read the test file and write it to the process's stdin
    let mut input_file = File::open(input_path).unwrap();
    let mut buffer = [0u8; 1024];
    loop {
        let bytes_read = input_file.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }
        
        let bytes_written = unsafe {
            process_write_stdin(proc, buffer.as_ptr(), bytes_read)
        };
        assert!(bytes_written > 0);
    }
    
    // Close stdin to signal EOF
    unsafe { process_close(proc) };
    
    // Wait for the process to exit
    let exit_code = unsafe { process_wait(proc) };
    assert_eq!(exit_code, 0);
    
    // Verify that the output file exists and matches the input
    assert!(output_path.exists());
    let output = std::fs::read_to_string(output_path).unwrap();
    assert_eq!(output, "Hello, world!\nThis is a test file.\n");
}

// Helper function to create a test file
fn create_test_file(path: &Path, content: &str) {
    let mut file = File::create(path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
} 