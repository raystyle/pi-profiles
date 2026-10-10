#!/usr/bin/env rust-script
//! name: blk_forge
//! description: AES 分组刀 (block knife) for fixed-IV / ECB oracle work - split a base64 ciphertext blob into 16-byte blocks (hex + ascii), flag duplicate blocks (ECB detection), extract chosen blocks, concatenate blobs, and XOR equal-length blobs (CBC IV rebuild), so an encryption oracle's ciphertext can be reassembled into a forged cookie without the key.
//! version: 1.0.0
//! args: blocks <b64> | dupes <b64> | take <b64> --n 3,4 | cat <b64>... | xor <b64> <b64> | selftest
//! keywords: crypto, aes, cbc, ecb, block, oracle, base64, xor, cookie, forge, knife
//!
//! ```cargo
//! [dependencies]
//! base64 = "0.22"
//! ```

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use pi_rust_lib::serde_json::json;
use std::collections::HashMap;

const BLOCK: usize = 16;

fn decode(s: &str) -> Result<Vec<u8>, String> {
    let mut t = s.trim().trim_matches('"').to_string();
    while t.ends_with('%') || t.contains("%3d") || t.contains("%3D") {
        t = t.replace("%3d", "=").replace("%3D", "=");
        break;
    }
    let pad = (4 - t.len() % 4) % 4;
    for _ in 0..pad {
        t.push('=');
    }
    STANDARD.decode(t).map_err(|e| format!("base64 decode: {e}"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn ascii(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| if (0x20..0x7f).contains(b) { *b as char } else { '.' })
        .collect()
}

fn chunk(data: &[u8]) -> Vec<&[u8]> {
    data.chunks(BLOCK).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        pi_rust_lib::report::failure(
            "blk_forge",
            "usage: blocks <b64> | dupes <b64> | take <b64> --n 3,4 | cat <b64>... | xor <b64> <b64> | selftest",
            "pass a subcommand",
        );
        std::process::exit(2);
    }
    let cmd = args[0].as_str();
    let rest: Vec<String> = args[1..].to_vec();
    let mut nums: Vec<usize> = Vec::new();
    let mut blobs: Vec<String> = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        if rest[i] == "--n" || rest[i] == "--blocks" {
            i += 1;
            if let Some(v) = rest.get(i) {
                for part in v.split(',') {
                    for r in part.split('-') {
                        if let Ok(n) = r.trim().parse::<usize>() {
                            nums.push(n);
                        }
                    }
                }
            }
        } else {
            blobs.push(rest[i].clone());
        }
        i += 1;
    }

    match cmd {
        "selftest" => {
            let a = vec![0x41u8; BLOCK];
            let b = vec![0x00u8; BLOCK];
            let x = a.iter().zip(b.iter()).map(|(p, q)| p ^ q).collect::<Vec<u8>>();
            let ok = x.iter().all(|v| *v == 0x41) && hex(&x) == "41".repeat(BLOCK);
            let dup_blob = STANDARD.encode([a.clone(), a.clone()].concat());
            let dup_bytes = STANDARD.decode(&dup_blob).unwrap_or_default();
            let parts = chunk(&dup_bytes);
            let ok2 = parts.len() == 2 && parts[0] == parts[1];
            let unpadded = STANDARD.encode(b"hello").trim_end_matches('=').to_string();
            let ok3 = decode(&unpadded).map(|d| d == b"hello").unwrap_or(false)
                && decode(&STANDARD.encode(b"hi").replace('=', "%3d")).map(|d| d == b"hi").unwrap_or(false);
            if ok && ok2 && ok3 {
                pi_rust_lib::report::success(
                    "blk_forge",
                    json!({"selftest": {"xor": ok, "cat": ok2, "urlencoded_pad": ok3}}),
                    "run blocks/dupes/take on a real ciphertext",
                )
                .expect("report");
            } else {
                pi_rust_lib::report::failure(
                    "blk_forge",
                    "selftest failed",
                    "inspect the block helpers",
                );
                std::process::exit(1);
            }
        }
        "blocks" | "dupes" => {
            let data = match blobs.first().map(|s| decode(s)) {
                Some(Ok(d)) => d,
                Some(Err(e)) => {
                    pi_rust_lib::report::failure("blk_forge", &e, "pass a valid base64 blob");
                    std::process::exit(1);
                }
                None => {
                    pi_rust_lib::report::failure("blk_forge", "missing base64 argument", "pass a blob");
                    std::process::exit(1);
                }
            };
            let parts = chunk(&data);
            let listing: Vec<_> = parts
                .iter()
                .enumerate()
                .map(|(i, b)| json!({"n": i + 1, "len": b.len(), "hex": hex(b), "ascii": ascii(b)}))
                .collect();
            let mut seen: HashMap<String, Vec<usize>> = HashMap::new();
            for (i, b) in parts.iter().enumerate() {
                seen.entry(hex(b)).or_default().push(i + 1);
            }
            let dupes: Vec<_> = seen
                .iter()
                .filter(|(_, v)| v.len() > 1)
                .map(|(h, v)| json!({"hex": h, "blocks": v}))
                .collect();
            let data_out = json!({
                "bytes": data.len(),
                "block_count": parts.len(),
                "blocks": if cmd == "dupes" { json!([]) } else { json!(listing) },
                "duplicate_blocks": dupes,
            });
            pi_rust_lib::report::success("blk_forge", data_out, "duplicate blocks present means ECB mode").expect("report");
        }
        "take" => {
            let data = match blobs.first().map(|s| decode(s)) {
                Some(Ok(d)) => d,
                Some(Err(e)) => {
                    pi_rust_lib::report::failure("blk_forge", &e, "pass a valid base64 blob");
                    std::process::exit(1);
                }
                None => {
                    pi_rust_lib::report::failure("blk_forge", "missing base64 argument", "pass a blob");
                    std::process::exit(1);
                }
            };
            if nums.is_empty() {
                pi_rust_lib::report::failure("blk_forge", "missing --n block list", "pass --n 3,4 (1-indexed)");
                std::process::exit(2);
            }
            let parts = chunk(&data);
            let mut out: Vec<u8> = Vec::new();
            let mut chosen: Vec<String> = Vec::new();
            for n in &nums {
                match parts.get(n - 1) {
                    Some(b) => {
                        out.extend_from_slice(b);
                        chosen.push(format!("{n}:{}", hex(b)));
                    }
                    None => {
                        pi_rust_lib::report::failure(
                            "blk_forge",
                            &format!("block {n} out of range (have {})", parts.len()),
                            "narrow the --n list",
                        );
                        std::process::exit(1);
                    }
                }
            }
            pi_rust_lib::report::success(
                "blk_forge",
                json!({"out_b64": STANDARD.encode(&out), "bytes": out.len(), "chosen": chosen}),
                "use out_b64 as the forged cookie value",
            )
            .expect("report");
        }
        "cat" => {
            let mut out: Vec<u8> = Vec::new();
            for b in &blobs {
                match decode(b) {
                    Ok(d) => out.extend_from_slice(&d),
                    Err(e) => {
                        pi_rust_lib::report::failure("blk_forge", &e, "pass valid base64 blobs");
                        std::process::exit(1);
                    }
                }
            }
            pi_rust_lib::report::success(
                "blk_forge",
                json!({"out_b64": STANDARD.encode(&out), "bytes": out.len(), "parts": blobs.len()}),
                "use out_b64 where the joined blob is needed",
            )
            .expect("report");
        }
        "xor" => {
            if blobs.len() < 2 {
                pi_rust_lib::report::failure("blk_forge", "xor needs two blobs", "pass <b64> <b64>");
                std::process::exit(2);
            }
            let a = decode(&blobs[0]);
            let b = decode(&blobs[1]);
            match (a, b) {
                (Ok(a), Ok(b)) => {
                    if a.len() != b.len() {
                        pi_rust_lib::report::failure(
                            "blk_forge",
                            &format!("length mismatch {} vs {}", a.len(), b.len()),
                            "xor equal-length blobs",
                        );
                        std::process::exit(1);
                    }
                    let x: Vec<u8> = a.iter().zip(b.iter()).map(|(p, q)| p ^ q).collect();
                    pi_rust_lib::report::success(
                        "blk_forge",
                        json!({"out_b64": STANDARD.encode(&x), "hex": hex(&x), "ascii": ascii(&x), "bytes": x.len()}),
                        "feed out_b64 into the next forge step",
                    )
                    .expect("report");
                }
                (Err(e), _) | (_, Err(e)) => {
                    pi_rust_lib::report::failure("blk_forge", &e, "pass valid base64 blobs");
                    std::process::exit(1);
                }
            }
        }
        other => {
            pi_rust_lib::report::failure(
                "blk_forge",
                &format!("unknown subcommand {other}"),
                "use blocks|dupes|take|cat|xor|selftest",
            );
            std::process::exit(2);
        }
    }
}
