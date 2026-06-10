use std::process::Command;
use std::env;

fn main() {
    // Re-run the build script if HEAD changes
    println!("cargo:rerun-if-changed=../.git/HEAD");

    let default_version = format!("v{}", env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "Unknown".to_string()));

    let output = Command::new("git")
        .args(&["describe", "--tags", "--abbrev=0"])
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
