//! Sets `GAME_VERSION` for the compiler so every build can identify itself:
//! the exact git tag if HEAD is tagged, otherwise the short SHA (plus `-dirty`
//! when the working tree has uncommitted changes). CI may override it by
//! exporting `GAME_VERSION`.
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (!s.is_empty()).then_some(s)
}

fn main() {
    println!("cargo:rerun-if-env-changed=GAME_VERSION");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
    println!("cargo:rerun-if-changed=.git/index");

    let version = std::env::var("GAME_VERSION").ok().or_else(|| {
        git(&["describe", "--tags", "--exact-match"]).or_else(|| {
            let sha = git(&["rev-parse", "--short", "HEAD"])?;
            let dirty = git(&["status", "--porcelain", "--untracked-files=no"]).is_some();
            Some(if dirty { format!("{sha}-dirty") } else { sha })
        })
    });
    println!(
        "cargo:rustc-env=GAME_VERSION={}",
        version.unwrap_or_else(|| "unknown".into())
    );
}
