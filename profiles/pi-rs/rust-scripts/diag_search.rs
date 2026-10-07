#!/usr/bin/env rust-script
//! name: diag_search
//! description: Search pi and evcxr for linker/env config
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
    println!("--- pi repo: evcxr/rust_repl/mold refs ---");
    print!("{}", sh("grep -rIn --exclude-dir=node_modules --exclude-dir=.git -e evcxr -e rust_repl -e mold /mnt/wsl/repos/pi 2>/dev/null | head -40"));
    println!("--- evcxr: env/config/linker refs ---");
    print!("{}", sh("grep -rIn -e 'EVCXR' -e 'env::var' /home/ray/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/evcxr-0.22.0/src/ | head -40"));
    println!("--- mold -run wrapper lookup ---");
    print!("{}", sh("mold -run /bin/true 2>&1; echo exit=$?; strings /home/ray/.cargo/bin/mold | grep -i -e 'mold-wrapper' -e 'MOLD_' | head"));
}
