use std::env;
use std::process::Command;

fn main() {
    // Re-run the build script if HEAD changes or if GIT_TAG env var changes
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-env-changed=GIT_TAG");

    let default_version = format!(
        "v{}",
        env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "Unknown".to_string())
    );

    // 1. Check if GIT_TAG env var is provided
    if let Ok(tag) = env::var("GIT_TAG") {
        if !tag.is_empty() {
            println!("cargo:rustc-env=GIT_TAG={}", tag);
            return;
        }
    }

    // 2. Fallback to git command
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=GIT_TAG={}", tag);
        } else {
            println!("cargo:rustc-env=GIT_TAG={}", default_version);
        }
    } else {
        println!("cargo:rustc-env=GIT_TAG={}", default_version);
    }
}
