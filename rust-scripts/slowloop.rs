#!/usr/bin/env rust-script
//! name: slowloop
//! description: Probe script: sleeps 60 seconds in main
//! version: 1.0.0
//! keywords: probe, sleep, timeout
//!
//! ```cargo
//! ```
use std::thread;
use std::time::Duration;

fn main() {
    println!("slowloop start");
    thread::sleep(Duration::from_secs(60));
    println!("slowloop done");
}
