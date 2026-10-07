#!/usr/bin/env rust-script
//! name: collatz
//! description: Compute the Collatz step count for a positive integer n
//! version: 1.0.0
//! args: <n>
//! keywords: collatz, math, sequence
//!
//! ```cargo
//! ```
use std::env;

fn main() {
    let arg = match env::args().nth(1) {
        Some(a) => a,
        None => {
            eprintln!("usage: collatz <n>");
            std::process::exit(2);
        }
    };

    let mut n: u128 = match arg.parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("n must be a positive integer, got {arg:?}");
            std::process::exit(2);
        }
    };

    if n == 0 {
        eprintln!("n must be >= 1");
        std::process::exit(2);
    }

    let mut steps: u64 = 0;
    let mut peak: u128 = n;
    while n != 1 {
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        if n > peak {
            peak = n;
        }
        steps += 1;
    }

    println!("n={arg} steps={steps} peak={peak}");
}
