# Cross-Compilation Guide

This document provides detailed instructions for building the BetaHub Process Wrapper library for multiple platforms.

## Prerequisites

Before you begin, make sure you have:

1. Rust and Cargo installed (https://rustup.rs/)
2. Required targets installed via rustup
3. Platform-specific tools as needed

## Installing Rust Targets

```bash
# Install all targets
rustup target add x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu x86_64-pc-windows-msvc

# Or install them individually
rustup target add x86_64-apple-darwin       # macOS (Intel)
rustup target add aarch64-apple-darwin      # macOS (Apple Silicon)
rustup target add x86_64-unknown-linux-gnu  # Linux (64-bit Intel)
rustup target add x86_64-pc-windows-msvc    # Windows (MSVC)
rustup target add x86_64-pc-windows-gnu     # Windows (GNU/MinGW)
```

## Building for macOS

### macOS Intel (x86_64)

```bash
cargo build --release --target x86_64-apple-darwin
```

The library will be at `target/x86_64-apple-darwin/release/libbetahub_process_wrapper.dylib`.

### macOS Apple Silicon (ARM64)

```bash
cargo build --release --target aarch64-apple-darwin
```

The library will be at `target/aarch64-apple-darwin/release/libbetahub_process_wrapper.dylib`.

### Universal Binary

To create a universal binary that works on both Intel and Apple Silicon Macs:

```bash
# Build for both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create universal binary
mkdir -p dist/macos
lipo -create \
  target/x86_64-apple-darwin/release/libbetahub_process_wrapper.dylib \
  target/aarch64-apple-darwin/release/libbetahub_process_wrapper.dylib \
  -output dist/macos/libbetahub_process_wrapper.dylib
```

## Building for Linux

### Linux (x86_64)

```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

The library will be at `target/x86_64-unknown-linux-gnu/release/libbetahub_process_wrapper.so`.

## Building for Windows

### Windows MSVC (x86_64)

```bash
cargo build --release --target x86_64-pc-windows-msvc
```

The library will be at:
- `target/x86_64-pc-windows-msvc/release/betahub_process_wrapper.dll` (Dynamic library)
- `target/x86_64-pc-windows-msvc/release/betahub_process_wrapper.lib` (Import library)

### Windows GNU/MinGW (x86_64)

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

The library will be at `target/x86_64-pc-windows-gnu/release/betahub_process_wrapper.dll`.

## Using the Build Script

For convenience, you can use the included build script:

```bash
# Make the script executable
chmod +x build-all.sh

# Build for your current platform
./build-all.sh

# Build for a specific platform
./build-all.sh --macos
./build-all.sh --linux
./build-all.sh --windows

# Build for all platforms (requires cross-compilation setup)
./build-all.sh --all
```

## Cross-Compilation Setup

### Linux to Windows

To build Windows binaries from Linux, install the MinGW toolchain:

```bash
# Ubuntu/Debian
sudo apt install mingw-w64

# Then add the target
rustup target add x86_64-pc-windows-gnu
```

### macOS to Linux

To build Linux binaries from macOS, you might need a cross-compilation environment like Docker:

```bash
# Example using Docker
docker run --rm -v "$(pwd)":/root/src -w /root/src rust:latest \
  bash -c "rustup target add x86_64-unknown-linux-gnu && cargo build --release --target x86_64-unknown-linux-gnu"
```

## Integration with C# (Unity)

For Unity integration, you'll need to place the appropriate library in the Unity project:

1. macOS: Place `libbetahub_process_wrapper.dylib` in `Assets/Plugins/macOS/`
2. Linux: Place `libbetahub_process_wrapper.so` in `Assets/Plugins/Linux/`
3. Windows: Place `betahub_process_wrapper.dll` in `Assets/Plugins/Windows/x86_64/`

## Troubleshooting

### Common Issues

1. **Missing linker**: Make sure you have the appropriate linker for your target platform.

2. **Library not found**: Check that you're using the correct library name for each platform:
   - macOS: `libbetahub_process_wrapper.dylib`
   - Linux: `libbetahub_process_wrapper.so`
   - Windows: `betahub_process_wrapper.dll`

3. **Symbol not found**: Ensure you're using the correct calling convention (`extern "C"`) and function signatures.

### Verifying the Build

You can verify the build for each platform using appropriate tools:

```bash
# macOS
file dist/macos/libbetahub_process_wrapper.dylib
otool -L dist/macos/libbetahub_process_wrapper.dylib

# Linux
file dist/linux/libbetahub_process_wrapper.so
ldd dist/linux/libbetahub_process_wrapper.so

# Windows (if on Windows)
dumpbin /EXPORTS dist/windows/betahub_process_wrapper.dll
``` 