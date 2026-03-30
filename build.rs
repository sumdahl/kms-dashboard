use std::process::Command;

fn main() {
    // Tell Cargo to rerun this script if your CSS or templates change
    println!("cargo:rerun-if-changed=src/styles/input.css");
    println!("cargo:rerun-if-changed=templates/");

    // Trigger the npm build
    let status = Command::new("npm")
        .args(["run", "build:css"])
        .status()
        .expect("Failed to run npm build:css");

    if !status.success() {
        panic!("CSS build failed");
    }
}
