# Logging Optimization Guide

This document explains the logging configuration optimizations implemented for the Rust RTS game to provide optimal performance in release builds while maintaining full logging capabilities during development.

## Overview

The game now uses **conditional compilation** to completely remove logging overhead in release builds while preserving full logging functionality in debug builds.

## Build Configurations

### Debug Builds (Default)
- **Command**: `cargo run` or `cargo run --features logging`
- **Logging**: Full tracing with `info!`, `debug!`, `warn!`, `error!` macros
- **Dependencies**: Includes `tracing-subscriber` and `tracing-flame`
- **Output**: Shows detailed logs with configurable levels
- **Startup message**: "🚀 Starting RTS Game... (Debug build with logging)"

### Release Builds
- **Command**: `cargo run --release --no-default-features`
- **Logging**: Completely disabled - zero runtime overhead
- **Dependencies**: Excludes logging dependencies entirely
- **Output**: Minimal console output
- **Startup message**: "🚀 Starting RTS Game... (Release build)"

## Performance Benefits

### Debug Builds
- Full logging for development and debugging
- Configurable log levels via `RUST_LOG` environment variable
- Asset warnings filtered to reduce noise

### Release Builds
- **Zero logging overhead**: All logging code removed at compile time
- **Smaller binary size**: Logging dependencies and strings stripped
- **Faster execution**: No log level checks or string formatting
- **Optimal performance**: Maximum FPS for production gameplay

## Usage Examples

```bash
# Debug build with full logging
cargo run

# Release build with no logging (maximum performance)
cargo run --release --no-default-features

# Debug build with custom log level
RUST_LOG=debug cargo run

# Profiling build (inherits from release but keeps debug info)
cargo build --profile profiling
```

## Technical Implementation

1. **Conditional Features**: Uses Cargo features to conditionally include logging dependencies
2. **Conditional Compilation**: Uses `#[cfg()]` attributes to exclude logging initialization in release
3. **Build Script**: `build.rs` sets compile-time flags based on build profile
4. **Bevy Integration**: Configured to work seamlessly with Bevy's tracing system

## File Structure

- `Cargo.toml`: Feature definitions and conditional dependencies
- `build.rs`: Build script for compile-time optimization flags
- `src/main.rs`: Conditional logging initialization
- `LOGGING_OPTIMIZATION.md`: This documentation

## Customization

To adjust log levels for debug builds, modify the `RUST_LOG` environment variable or update `main.rs`:

```rust
// In main.rs, modify this line for different default log levels:
std::env::set_var("RUST_LOG", "info,bevy_gltf=warn,bevy_ui::layout=error");
```

## Binary Size Comparison

The optimized release build will be significantly smaller due to:
- Removed logging dependencies (`tracing-subscriber`, `tracing-flame`)
- Stripped logging strings and formatting code
- Eliminated log level checking logic

This optimization provides the best of both worlds: comprehensive logging during development and maximum performance in release builds.