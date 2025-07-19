use std::env;
use std::path::Path;

fn main() {
    // Check if we're on Windows
    if cfg!(target_os = "windows") {
        // Look for Vosk libraries in the project's vosk-libs directory
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let project_root = Path::new(&manifest_dir).parent().unwrap().parent().unwrap();
        let vosk_lib_dir = project_root.join("vosk-libs").join("lib");
        
        if vosk_lib_dir.exists() {
            println!("cargo:rustc-link-search=native={}", vosk_lib_dir.display());
            println!("cargo:rustc-link-lib=dylib=libvosk");
            
            // Tell cargo to rerun this build script if the library directory changes
            println!("cargo:rerun-if-changed={}", vosk_lib_dir.display());
        } else {
            // Fallback: try to find libraries in system PATH or common locations
            println!("cargo:rustc-link-lib=dylib=libvosk");
            
            // Check if we can find the library in common Windows locations
            let possible_paths = [
                "C:\\Program Files\\vosk",
                "C:\\vosk",
                &env::var("VOSK_LIB_PATH").unwrap_or_default(),
            ];
            
            for path in &possible_paths {
                if !path.is_empty() && Path::new(path).exists() {
                    println!("cargo:rustc-link-search=native={}", path);
                    break;
                }
            }
        }
    } else {
        // For non-Windows platforms, use the standard library name
        println!("cargo:rustc-link-lib=dylib=vosk");
    }
    
    // Tell cargo to rerun this build script if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
} 