#!/usr/bin/env rust-script
//! name: labkit_sync
//! description: Mirror the repo labkit/ archive into the runnable global scripts dir for sessions whose catalog predates the labkit-scan build - copies only new/changed *.rs and reports each action.
//! version: 1.0.0
//! args: [--from DIR] [--to DIR] [--dry-run]
//! keywords: labkit, sync, scripts, mirror, install, catalog
//!
//! ```cargo
//! [dependencies]
//! ```

use pi_rust_lib::serde_json::{json, Value};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let cwd = std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| ".".into());
    let mut from = format!("{cwd}/labkit");
    let mut to = format!("{home}/.pi-rs/agent/rust-scripts");
    let mut dry = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--from" => {
                i += 1;
                if i < args.len() {
                    from = args[i].clone();
                }
            }
            "--to" => {
                i += 1;
                if i < args.len() {
                    to = args[i].clone();
                }
            }
            "--dry-run" => dry = true,
            _ => {}
        }
        i += 1;
    }

    let entries = match std::fs::read_dir(&from) {
        Ok(e) => e,
        Err(e) => {
            pi_rust_lib::report::failure("labkit_sync", &format!("cannot read {from}: {e}"), "pass --from <labkit dir>");
            std::process::exit(1);
        }
    };
    let _ = std::fs::create_dir_all(&to);

    let mut rows: Vec<Value> = Vec::new();
    let mut copied = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("rs") {
            continue;
        }
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let src = std::fs::read_to_string(&path).unwrap_or_default();
        let dest = format!("{to}/{name}");
        let action = match std::fs::read_to_string(&dest).ok() {
            Some(existing) if existing == src => "unchanged",
            _ => {
                if !dry {
                    let _ = std::fs::write(&dest, &src);
                }
                copied += 1;
                "copied"
            }
        };
        rows.push(json!({ "name": name, "action": action }));
    }

    let data = json!({
        "from": from,
        "to": to,
        "dry_run": dry,
        "copied": copied,
        "files": rows,
    });
    let next = if dry {
        "rerun without --dry-run to apply"
    } else {
        "labkit scripts are runnable by name as global scripts now"
    };
    pi_rust_lib::report::success("labkit_sync", data, next).expect("report success");
}
