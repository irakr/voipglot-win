use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Build Tauri
    tauri_build::build();
    
    // Copy native dependencies to the output directory
    copy_native_dependencies();
}

fn copy_native_dependencies() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    // Create resources directory structure for bundling
    let resources_dir = Path::new(&manifest_dir).join("resources");
    let models_dir = resources_dir.join("models");
    
    // Create empty resources directories to prevent bundling errors
    if !resources_dir.exists() {
        fs::create_dir_all(&resources_dir).expect("Failed to create resources directory");
        println!("cargo:warning=Created empty resources directory");
    }
    
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir).expect("Failed to create resources/models directory");
        println!("cargo:warning=Created empty resources/models directory");
    }
} 