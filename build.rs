fn main() {
    // Notify cargo to rerun this script if the build script itself changes
    println!("cargo:rerun-if-changed=build.rs");
    
    // Platform-specific configurations
    #[cfg(target_os = "windows")]
    {
        // Ensure the library is linked with the appropriate Windows libraries
        println!("cargo:rustc-link-lib=dylib=kernel32");
        println!("cargo:rustc-link-lib=dylib=user32");
    }
    
    #[cfg(target_os = "macos")]
    {
        // No special configuration needed for macOS at the moment
        // For future reference, macOS-specific linking would go here
    }
    
    #[cfg(target_os = "linux")]
    {
        // Link against the appropriate Linux libraries if needed
    }
} 