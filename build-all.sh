#!/bin/bash
set -e

# Output directory
OUTPUT_DIR="dist"
mkdir -p $OUTPUT_DIR

# Function to print status
print_status() {
  echo "========================================"
  echo "  $1"
  echo "========================================"
}

# Build for macOS (universal binary)
build_macos() {
  print_status "Building for macOS (Universal Binary)"
  
  # Make sure targets are installed
  rustup target add x86_64-apple-darwin aarch64-apple-darwin
  
  # Build for Intel
  print_status "Building for macOS (Intel x86_64)"
  cargo build --release --target x86_64-apple-darwin
  
  # Build for Apple Silicon
  print_status "Building for macOS (Apple Silicon ARM64)"
  cargo build --release --target aarch64-apple-darwin
  
  # Create universal binary
  print_status "Creating Universal Binary"
  mkdir -p $OUTPUT_DIR/macos
  lipo -create \
    target/x86_64-apple-darwin/release/libbetahub_process_wrapper.dylib \
    target/aarch64-apple-darwin/release/libbetahub_process_wrapper.dylib \
    -output $OUTPUT_DIR/macos/libbetahub_process_wrapper.dylib
  
  echo "macOS binary saved to $OUTPUT_DIR/macos/libbetahub_process_wrapper.dylib"
}

# Build for Linux (x86_64)
build_linux() {
  print_status "Building for Linux (x86_64)"
  
  # Make sure target is installed
  rustup target add x86_64-unknown-linux-gnu
  
  # Build for Linux
  cargo build --release --target x86_64-unknown-linux-gnu
  
  # Copy to output directory
  mkdir -p $OUTPUT_DIR/linux
  cp target/x86_64-unknown-linux-gnu/release/libbetahub_process_wrapper.so $OUTPUT_DIR/linux/
  
  echo "Linux binary saved to $OUTPUT_DIR/linux/libbetahub_process_wrapper.so"
}

# Build for Windows (x86_64)
build_windows() {
  print_status "Building for Windows (x86_64)"
  
  # Try to build using MSVC toolchain if available
  if rustup target list --installed | grep -q "x86_64-pc-windows-msvc"; then
    print_status "Building with MSVC toolchain"
    rustup target add x86_64-pc-windows-msvc
    cargo build --release --target x86_64-pc-windows-msvc
    
    mkdir -p $OUTPUT_DIR/windows
    cp target/x86_64-pc-windows-msvc/release/betahub_process_wrapper.dll $OUTPUT_DIR/windows/
    cp target/x86_64-pc-windows-msvc/release/betahub_process_wrapper.lib $OUTPUT_DIR/windows/
    
    echo "Windows binary saved to $OUTPUT_DIR/windows/betahub_process_wrapper.dll"
  # Fall back to GNU toolchain
  elif rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
    print_status "Building with GNU toolchain"
    rustup target add x86_64-pc-windows-gnu
    cargo build --release --target x86_64-pc-windows-gnu
    
    mkdir -p $OUTPUT_DIR/windows
    cp target/x86_64-pc-windows-gnu/release/betahub_process_wrapper.dll $OUTPUT_DIR/windows/
    
    echo "Windows binary saved to $OUTPUT_DIR/windows/betahub_process_wrapper.dll"
  else
    echo "Error: No Windows toolchain found. Please install either x86_64-pc-windows-msvc or x86_64-pc-windows-gnu"
    exit 1
  fi
}

# Detect platform
PLATFORM=$(uname)

if [ "$1" = "--all" ]; then
  # Try to build for all platforms (will only work with proper cross-compilation setup)
  build_macos
  build_linux
  build_windows
elif [ "$1" = "--macos" ]; then
  build_macos
elif [ "$1" = "--linux" ]; then
  build_linux
elif [ "$1" = "--windows" ]; then
  build_windows
else
  # Default: build for current platform
  if [ "$PLATFORM" = "Darwin" ]; then
    build_macos
  elif [ "$PLATFORM" = "Linux" ]; then
    build_linux
  else
    echo "Unsupported platform: $PLATFORM"
    echo "Please use one of the following flags: --macos, --linux, --windows, --all"
    exit 1
  fi
fi

print_status "Build completed!"
echo "All binaries are in the $OUTPUT_DIR directory" 