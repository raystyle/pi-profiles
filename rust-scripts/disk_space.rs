#!/usr/bin/env rust-script
//! name: disk_space
//! description: Report filesystem space, inode usage, and memory via df/free.
//! version: 1.0.0
//! keywords: disk, df, inodes, memory, space
//!
//! ```cargo
//! [dependencies]
//! ```
use pi_rust_lib::{report, serde_json};

fn main() {
    let mut blocks = Vec::new();
    for cmd in ["df -h", "df -i", "free -h"] {
        match pi_rust_lib::sh(cmd) {
            Ok(out) => blocks.push(format!("$ {cmd}\n{}", out.trim_end())),
            Err(err) => blocks.push(format!("$ {cmd}\n{err}")),
        }
    }
    report::success(
        "disk_space",
        serde_json::json!({ "report": blocks.join("\n\n") }),
        "drill into a large tree with dir_stats",
    )
    .expect("report success");
}
