use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    let vosk_path = env::var("VOSK_LIB_PATH")
        .or_else(|_| env::var("LIBRARY_PATH"))
        .unwrap_or_else(|_| "C:\\vosk".to_string());

    println!("cargo:rustc-link-search=native={}", vosk_path);
    println!("cargo:rustc-link-lib=dylib=vosk");
    
    // Re-run if VOSK path changes
    println!("cargo:rerun-if-env-changed=VOSK_LIB_PATH");
    println!("cargo:rerun-if-env-changed=LIBRARY_PATH");
    
    // Also check if the library file exists
    let lib_path = PathBuf::from(&vosk_path).join("libvosk.lib");
    if lib_path.exists() {
        println!("cargo:warning=VOSK library found at: {}", lib_path.display());
    } else {
        println!("cargo:warning=VOSK library not found at: {}", lib_path.display());
        println!("cargo:warning=Please ensure VOSK is installed and VOSK_LIB_PATH is set correctly");
    }
} 