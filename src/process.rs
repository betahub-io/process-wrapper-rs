use std::io::{self, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Process already finished")]
    ProcessFinished,
    
    #[error("Invalid process state")]
    InvalidState,
    
    #[error("Null pointer provided")]
    NullPointer,
}

pub type Result<T> = std::result::Result<T, ProcessError>;

/// Internal representation of a process
pub struct Process {
    /// The child process handle
    process: Option<Child>,
    
    /// Handle to the process's stdin
    stdin: Option<Arc<Mutex<std::process::ChildStdin>>>,
    
    /// Buffer for stderr output
    stderr_buffer: Arc<Mutex<Vec<u8>>>,
    
    /// Exit code if the process has finished
    exit_code: Option<i32>,
}

impl Process {
    /// Start a new process with the given command line
    /// 
    /// This is kept for backward compatibility
    pub fn new(cmd: &str) -> Result<Self> {
        let args: Vec<&str> = cmd.split_whitespace().collect();
        
        if args.is_empty() {
            return Err(ProcessError::InvalidState);
        }
        
        let program = args[0];
        let args = &args[1..];
        
        Self::new_with_args(program, args)
    }
    
    /// Start a new process with the given program path and arguments
    pub fn new_with_args(program: &str, args: &[&str]) -> Result<Self> {
        if program.is_empty() {
            return Err(ProcessError::InvalidState);
        }
        
        // Create the command
        let mut command = Command::new(program);
        command
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null()) // We don't care about stdout by default
            .stderr(Stdio::piped());
        
        // Spawn the process
        let mut child = command.spawn()?;
        
        // Take ownership of the I/O handles
        let stdin = child.stdin.take().map(|stdin| Arc::new(Mutex::new(stdin)));
        let stderr = child.stderr.take();
        
        // Create a buffer for stderr output
        let stderr_buffer = Arc::new(Mutex::new(Vec::new()));
        
        // If we have a stderr handle, spawn a thread to read from it
        if let Some(mut stderr) = stderr {
            let buffer = Arc::clone(&stderr_buffer);
            thread::spawn(move || {
                let mut buf = [0u8; 1024];
                loop {
                    match stderr.read(&mut buf) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            let mut buffer = buffer.lock().unwrap();
                            buffer.extend_from_slice(&buf[..n]);
                        }
                        Err(_) => break,
                    }
                }
            });
        }
        
        Ok(Process {
            process: Some(child),
            stdin,
            stderr_buffer,
            exit_code: None,
        })
    }
    
    /// Write data to the process's stdin
    pub fn write_stdin(&mut self, data: &[u8]) -> Result<usize> {
        if let Some(stdin) = &self.stdin {
            let mut stdin = stdin.lock().unwrap();
            stdin.write(data).map_err(ProcessError::Io)
        } else {
            Err(ProcessError::InvalidState)
        }
    }
    
    /// Read data from the stderr buffer
    pub fn read_stderr(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut stderr_buffer = self.stderr_buffer.lock().unwrap();
        
        let bytes_to_read = std::cmp::min(buf.len(), stderr_buffer.len());
        if bytes_to_read == 0 {
            return Ok(0);
        }
        
        buf[..bytes_to_read].copy_from_slice(&stderr_buffer[..bytes_to_read]);
        stderr_buffer.drain(..bytes_to_read);
        
        Ok(bytes_to_read)
    }
    
    /// Check if the process is still running
    pub fn is_running(&mut self) -> bool {
        if self.exit_code.is_some() {
            return false;
        }
        
        if let Some(process) = &mut self.process {
            match process.try_wait() {
                Ok(Some(status)) => {
                    self.exit_code = status.code();
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }
    
    /// Wait for the process to exit and return the exit code
    pub fn wait(&mut self) -> Result<i32> {
        // If we already have an exit code, return it
        if let Some(exit_code) = self.exit_code {
            return Ok(exit_code);
        }
        
        // If we have a process, wait for it
        if let Some(process) = &mut self.process {
            let status = process.wait()?;
            let exit_code = status.code().unwrap_or(-1);
            self.exit_code = Some(exit_code);
            Ok(exit_code)
        } else {
            Err(ProcessError::InvalidState)
        }
    }
    
    /// Close stdin, terminate the process, and clean up resources
    pub fn close(&mut self) -> Result<()> {
        // Drop stdin to close it
        self.stdin = None;
        
        // If the process is still running, try to terminate it
        if self.is_running() {
            if let Some(mut process) = self.process.take() {
                // Try to kill the process first
                let _ = process.kill();
                let status = process.wait()?;
                self.exit_code = status.code();
            }
        }
        
        Ok(())
    }
} 