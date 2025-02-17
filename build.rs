use std::process::Command;

fn main() {
    let stdout = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap()
        .stdout;

    let git_sha = String::from_utf8(stdout).unwrap();
    println!("cargo:rustc-env=GIT_SHA={}", git_sha.trim());
}
