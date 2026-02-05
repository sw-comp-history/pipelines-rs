//! Build script to capture build information.
//!
//! Sets environment variables at compile time:
//! - BUILD_HOST: hostname of the build machine
//! - BUILD_COMMIT: short git commit SHA
//! - BUILD_TIMESTAMP: ISO 8601 timestamp

use std::process::Command;

fn main() {
    // Get hostname
    let hostname = Command::new("hostname")
        .arg("-s")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Get git short SHA
    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Get ISO timestamp
    let timestamp = Command::new("date")
        .arg("-u")
        .arg("+%Y-%m-%dT%H:%M:%SZ")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Set environment variables for compilation
    println!("cargo:rustc-env=BUILD_HOST={}", hostname);
    println!("cargo:rustc-env=BUILD_COMMIT={}", commit);
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);

    // Rerun if git HEAD changes
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=build.rs");
}
