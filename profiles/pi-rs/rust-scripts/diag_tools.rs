#!/usr/bin/env rust-script
//! name: diag_tools
//! description: Check tooling and ability to obtain mold-wrapper.so
//! version: 1.0.0
//! keywords: diag
//!
//! ```cargo
//! ```
use std::process::Command;

fn sh(cmd: &str) -> String {
    let out = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

fn main() {
    println!("--- tooling ---");
    print!("{}", sh("for t in g++ cc clang++ curl wget git cmake lld ld.lld; do printf '%-8s ' $t; command -v $t || echo MISSING; done"));
    println!("--- dirs ---");
    print!("{}", sh("ls -ld /home/ray/.cargo/lib /home/ray/lib /usr/local/lib/mold /home/ray/.pi-rs/agent/bin 2>&1"));
    print!("{}", sh("ls /home/ray/.pi-rs/agent/bin 2>&1 | head"));
    println!("--- network github ---");
    print!("{}", sh("timeout 15 curl -sSI https://github.com 2>&1 | head -3; echo exit=$?"));
    println!("--- existing wrappers on disk ---");
    print!("{}", sh("find / -name 'mold-wrapper*' 2>/dev/null | head; find / -name 'mold-wrapper.so' 2>/dev/null | head"));
}
