#!/usr/bin/env rust-script
//! name: digits_demo
//! description: Print the first number in a string, or sleep N seconds with 'sleep N' (v2).
//! version: 1.0.1
//! args: <text> | sleep <secs>
//! keywords: demo, regex, bench
//!
//! ```cargo
//! ```
//! ```cargo
//! [dependencies]
//! regex = "1"
//! ```
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) == Some("sleep") {
        let secs: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(2);
        println!("sleeping {secs}s");
        std::thread::sleep(std::time::Duration::from_secs(secs));
        println!("awake");
        return;
    }
    let text = args.first().cloned().unwrap_or_else(|| "a1b".to_string());
    let m = regex::Regex::new(r"\d+")
        .unwrap()
        .find(&text)
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "none".into());
    println!("input={text} first_number={m}");
}
