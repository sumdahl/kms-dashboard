use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::process::Command;

fn main() {
    // Tell Cargo when to re-run this script
    println!("cargo:rerun-if-changed=src/styles/input.css");
    println!("cargo:rerun-if-changed=templates/");

    // 1. Run the build (Cross-platform friendly)
    let npm_cmd = if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    };

    let status = Command::new(npm_cmd)
        .args(["run", "build:css"])
        .status()
        .expect("Failed to run npm build:css");

    if !status.success() {
        panic!("CSS build failed");
    }

    // 2. Hash the output file
    let css_path = "static/css/output.css";

    // Safety check for fresh clones/builds
    if let Ok(css_content) = fs::read(css_path) {
        let mut hasher = DefaultHasher::new();
        css_content.hash(&mut hasher);
        let hash = hasher.finish();
        println!("cargo:rustc-env=CSS_VERSION={:x}", hash);
    } else {
        // Fallback if file isn't ready yet
        println!("cargo:rustc-env=CSS_VERSION=default");
    }
}
