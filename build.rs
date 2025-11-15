use std::env;

fn main() {
    // Set compile-time log level based on build profile
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    
    match profile.as_str() {
        "release" => {
            // Completely disable logging in release builds for maximum performance
            println!("cargo:rustc-cfg=feature=\"tracing_max_level_off\"");
            println!("cargo:rustc-cfg=feature=\"no_logging\"");
        }
        "profiling" => {
            // Minimal logging for profiling builds
            println!("cargo:rustc-cfg=feature=\"tracing_max_level_warn\"");
        }
        _ => {
            // Full logging for debug builds
            println!("cargo:rustc-cfg=feature=\"tracing_max_level_trace\"");
        }
    }
    
    println!("cargo:rerun-if-env-changed=PROFILE");
}