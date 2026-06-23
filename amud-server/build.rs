use std::env;
use std::process::Command;

fn is_release_tag(tag: &str) -> bool {
    let t = tag.trim().trim_start_matches('v');
    !t.is_empty()
        && t.split('.')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

fn normalize_release_tag(tag: &str) -> String {
    let trimmed = tag.trim();
    if trimmed.starts_with('v') {
        trimmed.to_string()
    } else {
        format!("v{}", trimmed)
    }
}

fn git_latest_tag() -> Option<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if is_release_tag(&tag) {
        Some(normalize_release_tag(&tag))
    } else {
        None
    }
}

fn main() {
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-env-changed=GIT_TAG");

    let default_version = normalize_release_tag(
        &env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "Unknown".to_string()),
    );

    if let Ok(tag) = env::var("GIT_TAG") {
        if is_release_tag(&tag) {
            println!("cargo:rustc-env=GIT_TAG={}", normalize_release_tag(&tag));
            return;
        }
    }

    if let Some(tag) = git_latest_tag() {
        println!("cargo:rustc-env=GIT_TAG={}", tag);
    } else {
        println!("cargo:rustc-env=GIT_TAG={}", default_version);
    }
}
