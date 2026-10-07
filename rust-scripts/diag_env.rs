#!/usr/bin/env rust-script
//! name: diag_env
//! description: Print linker-related env vars and configs
//! version: 1.0.1
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
    for k in ["CARGO_HOME", "CARGO_TARGET_DIR", "CARGO_BUILD_TARGET", "CARGO_BUILD_RUSTFLAGS", "RUSTC_LINKER", "CARGO_ENCODED_RUSTFLAGS"] {
        println!("{k}={:?}", std::env::var(k));
    }
    println!("--- cargo configs ---");
    print!("{}", sh("for f in /home/ray/.cargo/config /home/ray/.cargo/config.toml /mnt/.cargo/config.toml /mnt/wsl/.cargo/config.toml /mnt/wsl/repos/.cargo/config.toml /mnt/wsl/repos/pi/.cargo/config.toml; do if [ -f $f ]; then echo \"== $f ==\"; cat $f; fi; done"));
    println!("--- mold binary ---");
    print!("{}", sh("file /home/ray/.cargo/bin/mold; ls -l /home/ray/.cargo/bin/ | grep -i mold; /home/ray/.cargo/bin/mold --version 2>&1 | head -3"));
    println!("--- grep mold ---");
    print!("{}", sh("grep -rIl --exclude-dir=node_modules --exclude-dir=.git mold /home/ray/.cargo /home/ray/.config /etc 2>/dev/null | head -20"));
    println!("--- find mold-wrapper ---");
    print!("{}", sh("find /home/ray /usr -name 'mold-wrapper.so' 2>/dev/null | head"));
}
