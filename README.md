# BetaHub Process Wrapper Library

A Rust FFI library for controlling system processes from external applications. This library provides a C ABI for easy integration with C# (Unity) and other languages.

## Features

- Spawn and manage system processes
- Write data to process's stdin (for piping data)
- Capture stderr output for error handling
- Process status monitoring and cleanup
- Thread-safe I/O operations
- Cross-platform (Windows, macOS, Linux)

## Build

```bash
# Build the library
cargo build --release

# Run tests
cargo test
```

### Build Script

This repository includes a build script that automates the process of building for all supported platforms:

```bash
# Build for your current platform
./build-all.sh

# Build for a specific platform
./build-all.sh --macos
./build-all.sh --linux
./build-all.sh --windows

# Build for all platforms (requires cross-compilation setup)
./build-all.sh --all
```

The script will create a `dist` directory with subdirectories for each platform containing the compiled libraries.

## Cross-Compilation

This library supports multiple target platforms. Follow these instructions to build for different targets:

### Prerequisites

First, install the Rust targets for cross-compilation:

```bash
# For macOS (Intel)
rustup target add x86_64-apple-darwin

# For macOS (Apple Silicon/ARM)
rustup target add aarch64-apple-darwin

# For Linux (64-bit Intel)
rustup target add x86_64-unknown-linux-gnu

# For Windows (64-bit Intel)
rustup target add x86_64-pc-windows-msvc  # If using MSVC toolchain
# OR
rustup target add x86_64-pc-windows-gnu   # If using MinGW toolchain
```

### Building for specific targets

```bash
# Build for macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# Build for macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Build for Linux (64-bit Intel)
cargo build --release --target x86_64-unknown-linux-gnu

# Build for Windows (64-bit Intel)
cargo build --release --target x86_64-pc-windows-msvc
# OR
cargo build --release --target x86_64-pc-windows-gnu
```

### Creating Universal Binary for macOS

To create a universal binary for macOS that works on both Intel and Apple Silicon:

```bash
# Build for both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create a universal binary using lipo
lipo -create \
  target/x86_64-apple-darwin/release/libbetahub_process_wrapper.dylib \
  target/aarch64-apple-darwin/release/libbetahub_process_wrapper.dylib \
  -output target/release/libbetahub_process_wrapper.dylib
```

### Additional Windows Build Notes

When building for Windows, you need to ensure the correct toolchain is installed:

- For MSVC target: Install Visual Studio or the Visual C++ Build Tools
- For GNU target: Install MinGW-w64

## C ABI Interface

The library exposes the following functions with C ABI:

```c
// Create and start a process (with combined command string)
void* process_start(const char* cmd);

// Create and start a process (with separate program and arguments)
void* process_start_with_args(const char* program, const char** args, size_t args_len);

// Write data to process's stdin
ssize_t process_write_stdin(void* proc, const uint8_t* data, size_t len);

// Read from process's stderr
ssize_t process_read_stderr(void* proc, uint8_t* buf, size_t len);

// Check if process is still running
int process_is_running(void* proc);

// Wait for process to exit
int process_wait(void* proc);

// Close stdin, terminate process, cleanup resources
void process_close(void* proc);
```

## Example Usage (C#)

```csharp
using System;
using System.Runtime.InteropServices;

public class ProcessWrapper
{
    // DllImport declarations
    [DllImport("betahub_process_wrapper", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr process_start(string cmd);
    
    [DllImport("betahub_process_wrapper", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr process_start_with_args(string program, string[] args, UIntPtr args_len);
    
    [DllImport("betahub_process_wrapper", CallingConvention = CallingConvention.Cdecl)]
    private static extern long process_write_stdin(IntPtr proc, byte[] data, UIntPtr len);
    
    [DllImport("betahub_process_wrapper", CallingConvention = CallingConvention.Cdecl)]
    private static extern long process_read_stderr(IntPtr proc, byte[] buf, UIntPtr len);
    
    [DllImport("betahub_process_wrapper", CallingConvention = CallingConvention.Cdecl)]
    private static extern int process_is_running(IntPtr proc);
    
    [DllImport("betahub_process_wrapper", CallingConvention = CallingConvention.Cdecl)]
    private static extern int process_wait(IntPtr proc);
    
    [DllImport("betahub_process_wrapper", CallingConvention = CallingConvention.Cdecl)]
    private static extern void process_close(IntPtr proc);
    
    // Example usage with ffmpeg (using combined command string)
    public static void EncodeFramesLegacy(byte[] frameData, string outputFile)
    {
        string cmd = $"ffmpeg -f rawvideo -pix_fmt rgb24 -s 1920x1080 -i pipe:0 -c:v libx264 -y {outputFile}";
        IntPtr proc = process_start(cmd);
        
        if (proc == IntPtr.Zero)
        {
            Console.WriteLine("Failed to start process");
            return;
        }
        
        try
        {
            // Write frame data
            long bytesWritten = process_write_stdin(proc, frameData, (UIntPtr)frameData.Length);
            
            // Read any error output
            byte[] errorBuf = new byte[1024];
            long bytesRead = process_read_stderr(proc, errorBuf, (UIntPtr)errorBuf.Length);
            if (bytesRead > 0)
            {
                string errorOutput = System.Text.Encoding.UTF8.GetString(errorBuf, 0, (int)bytesRead);
                Console.WriteLine($"Process stderr: {errorOutput}");
            }
            
            // Wait for the process to finish
            int exitCode = process_wait(proc);
            Console.WriteLine($"Process exited with code: {exitCode}");
        }
        finally
        {
            // Clean up resources
            process_close(proc);
        }
    }
    
    // Example usage with ffmpeg (using separate program and arguments)
    public static void EncodeFrames(byte[] frameData, string outputFile)
    {
        string program = "ffmpeg";
        string[] args = new string[] {
            "-f", "rawvideo",
            "-pix_fmt", "rgb24",
            "-s", "1920x1080",
            "-i", "pipe:0",
            "-c:v", "libx264",
            "-y", outputFile
        };
        
        IntPtr proc = process_start_with_args(program, args, (UIntPtr)args.Length);
        
        if (proc == IntPtr.Zero)
        {
            Console.WriteLine("Failed to start process");
            return;
        }
        
        try
        {
            // Write frame data
            long bytesWritten = process_write_stdin(proc, frameData, (UIntPtr)frameData.Length);
            
            // Read any error output
            byte[] errorBuf = new byte[1024];
            long bytesRead = process_read_stderr(proc, errorBuf, (UIntPtr)errorBuf.Length);
            if (bytesRead > 0)
            {
                string errorOutput = System.Text.Encoding.UTF8.GetString(errorBuf, 0, (int)bytesRead);
                Console.WriteLine($"Process stderr: {errorOutput}");
            }
            
            // Wait for the process to finish
            int exitCode = process_wait(proc);
            Console.WriteLine($"Process exited with code: {exitCode}");
        }
        finally
        {
            // Clean up resources
            process_close(proc);
        }
    }
}
```

## License

MIT 