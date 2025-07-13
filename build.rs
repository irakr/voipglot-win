use std::env;
use std::path::PathBuf;

fn main() {
    // Force static runtime library for all C++ dependencies
    env::set_var("CXXFLAGS", "/MT");
    env::set_var("CFLAGS", "/MT");
    env::set_var("CXXFLAGS_DEBUG", "/MTd");
    env::set_var("CFLAGS_DEBUG", "/MTd");
    env::set_var("CXXFLAGS_RELEASE", "/MT");
    env::set_var("CFLAGS_RELEASE", "/MT");
    
    // Tell cargo to look for shared libraries in the specified directory
    let vosk_path = env::var("VOSK_LIB_PATH")
        .or_else(|_| env::var("LIBRARY_PATH"))
        .unwrap_or_else(|_| "C:\\vosk".to_string());

    // Set environment variables for VOSK (similar to the PowerShell script)
    env::set_var("LIBRARY_PATH", &vosk_path);
    env::set_var("VOSK_LIB_PATH", &vosk_path);
    env::set_var("INCLUDE_PATH", &vosk_path);

    println!("cargo:rustc-link-search=native={}", vosk_path);
    
    // Try linking directly to the libvosk.lib file
    let lib_path = PathBuf::from(&vosk_path).join("libvosk.lib");
    if lib_path.exists() {
        println!("cargo:rustc-link-lib=static=libvosk");
    } else {
        // Fallback to dylib=vosk if libvosk.lib not found
        println!("cargo:rustc-link-lib=dylib=vosk");
    }
    
    // Re-run if VOSK path changes
    println!("cargo:rerun-if-env-changed=VOSK_LIB_PATH");
    println!("cargo:rerun-if-env-changed=LIBRARY_PATH");
    
    // Also check if the library file exists
    let lib_path_alt = PathBuf::from(&vosk_path).join("lib").join("libvosk.lib");
    let lib_path_win = PathBuf::from(&vosk_path).join("win64").join("libvosk.lib");
    
    if lib_path.exists() {
        println!("cargo:warning=VOSK library found at: {}", lib_path.display());
    } else if lib_path_alt.exists() {
        println!("cargo:warning=VOSK library found at: {}", lib_path_alt.display());
        println!("cargo:rustc-link-search=native={}", PathBuf::from(&vosk_path).join("lib").display());
    } else if lib_path_win.exists() {
        println!("cargo:warning=VOSK library found at: {}", lib_path_win.display());
        println!("cargo:rustc-link-search=native={}", PathBuf::from(&vosk_path).join("win64").display());
    } else {
        println!("cargo:warning=VOSK library not found at: {}", lib_path.display());
        println!("cargo:warning=Also checked: {}", lib_path_alt.display());
        println!("cargo:warning=Also checked: {}", lib_path_win.display());
        println!("cargo:warning=Please ensure VOSK is installed and VOSK_LIB_PATH is set correctly");
        println!("cargo:warning=You can download VOSK from: https://alphacephei.com/vosk/");
        println!("cargo:warning=Or set VOSK_LIB_PATH environment variable to point to the VOSK library directory");
    }
} 