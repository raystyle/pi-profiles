#!/usr/bin/env rust-script
//! name: md5_crack
//! description: 离线 MD5 口令破解件 - 对候选口令(词表文件或内联逗号表)逐个求 MD5 与目标哈希比对,可选按字符集做长度递增穷举,命中即回明文;单信封给出命中口令、尝试数、耗时与来源。离线凭证破解专用(哈希来自 cookie/DB 泄露等),不发任何网络请求。合法授权测试用途。
//! version: 1.1.0
//! args: <hash> (--wordlist FILE | --words 'a,b,c')... [--mask lower|digits|alnum|hexlower] [--max-len N] [--selftest]
//! keywords: 认证, 离线破解, md5, password-cracking, wordlist, 词表, 穷举, brute-force
//!
//! ```cargo
//! [dependencies]
//! md5 = "0.7"
//! ```

use pi_rust_lib::serde_json::{json, Value};
use pi_rust_lib::report;

/// Lowercase hex MD5 of a candidate string.
fn md5_hex(s: &str) -> String {
    format!("{:x}", md5::compute(s.as_bytes()))
}

fn charset_for(name: &str) -> Option<&'static [u8]> {
    match name {
        "lower" => Some(b"abcdefghijklmnopqrstuvwxyz"),
        "digits" => Some(b"0123456789"),
        "alnum" => Some(b"abcdefghijklmnopqrstuvwxyz0123456789"),
        "hexlower" => Some(b"0123456789abcdef"),
        _ => None,
    }
}

/// Odometer over the charset, length 1..=max_len, in order.
fn brute(charset: &[u8], max_len: usize, target: &str) -> Option<(String, u64)> {
    let cs = charset.len() as u64;
    let mut tried: u64 = 0;
    for len in 1..=max_len {
        let total = cs.pow(len as u32);
        let mut buf = vec![0u8; len];
        for idx in 0..total {
            let mut n = idx;
            for slot in (0..len).rev() {
                buf[slot] = charset[(n % cs) as usize];
                n /= cs;
            }
            let cand = std::str::from_utf8(&buf).expect("charset is ascii");
            tried += 1;
            if md5_hex(cand) == target {
                return Some((cand.to_string(), tried));
            }
        }
    }
    None
}

/// Wordlist pass first, optional mask brute force second. `elapsed` is timed
/// by the caller so both legs are covered by one number.
fn run(hash: &str, candidates: &[String], mask: Option<&str>, max_len: usize) -> Value {
    let start = std::time::Instant::now();
    let mut tried: u64 = 0;
    for cand in candidates {
        tried += 1;
        if md5_hex(cand) == hash {
            return json!({
                "hash": hash,
                "cracked": true,
                "password": cand,
                "source": "wordlist",
                "tried": tried,
                "elapsed_ms": start.elapsed().as_millis() as u64,
            });
        }
    }
    if let Some(mask) = mask {
        let cs = charset_for(mask).expect("charset validated before run");
        if let Some((pw, extra)) = brute(cs, max_len, hash) {
            return json!({
                "hash": hash,
                "cracked": true,
                "password": pw,
                "source": format!("brute:{mask}:<=len{max_len}"),
                "tried": tried + extra,
                "elapsed_ms": start.elapsed().as_millis() as u64,
            });
        }
        tried += (1..=max_len as u32).map(|l| (cs.len() as u64).pow(l)).sum::<u64>();
    }
    json!({
        "hash": hash,
        "cracked": false,
        "password": Value::Null,
        "source": Value::Null,
        "tried": tried,
        "elapsed_ms": start.elapsed().as_millis() as u64,
    })
}

fn selftest() {
    let checks = [
        ("5f4dcc3b5aa765d61d8327deb882cf99", "password"),
        ("e10adc3949ba59abbe56e057f20f883e", "123456"),
        ("827ccb0eea8a706c4c34a16891f84e7b", "12345"),
    ];
    let mut ok = true;
    let mut results = Vec::new();
    for (h, want) in checks {
        let got = run(h, &[want.to_string(), "not-it".to_string()], None, 0);
        let hit = got["cracked"] == json!(true) && got["password"] == json!(want);
        ok &= hit;
        results.push(json!({ "hash": h, "want": want, "ok": hit }));
    }
    // Brute leg must find md5("a") within the lowercase charset, length 1.
    let a = run("0cc175b9c0f1b6a831c399e269772661", &[], Some("lower"), 1);
    let brute_ok = a["cracked"] == json!(true) && a["password"] == json!("a");
    ok &= brute_ok;
    let payload = json!({ "ok": ok, "wordlist_checks": results, "brute_probe": a });
    if ok {
        report::success("md5_crack", payload, "cracking works; pass a real hash plus a wordlist").expect("report");
    } else {
        report::failure("md5_crack", &payload.to_string(), "fix the hash comparison");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }

    let mut hash: Option<String> = None;
    let mut wordlists: Vec<String> = Vec::new();
    let mut candidates: Vec<String> = Vec::new();
    let mut mask: Option<String> = None;
    let mut max_len: usize = 4;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--wordlist" => {
                i += 1;
                match args.get(i) {
                    Some(p) => wordlists.push(p.clone()),
                    None => {
                        report::failure("md5_crack", "--wordlist needs a path", "pass the wordlist file path");
                        return;
                    }
                }
            }
            "--words" => {
                i += 1;
                if let Some(s) = args.get(i) {
                    candidates.extend(s.split(',').map(|w| w.trim().to_string()).filter(|w| !w.is_empty()));
                }
            }
            "--mask" => {
                i += 1;
                match args.get(i) {
                    Some(m) if charset_for(m).is_some() => mask = Some(m.clone()),
                    Some(m) => {
                        report::failure(
                            "md5_crack",
                            &format!("unknown mask: {m}"),
                            "use --mask lower|digits|alnum|hexlower",
                        );
                        return;
                    }
                    None => {
                        report::failure("md5_crack", "--mask needs a charset name", "use --mask lower|digits|alnum|hexlower");
                        return;
                    }
                }
            }
            "--max-len" => {
                i += 1;
                max_len = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(4);
            }
            other if hash.is_none() && !other.starts_with('-') => hash = Some(other.to_lowercase()),
            other => {
                report::failure(
                    "md5_crack",
                    &format!("unknown arg: {other}"),
                    "usage: md5_crack <hash> (--wordlist FILE | --words 'a,b') [--mask lower] [--max-len N]",
                );
                return;
            }
        }
        i += 1;
    }

    let Some(hash) = hash else {
        report::failure(
            "md5_crack",
            "missing target hash",
            "usage: md5_crack <hash> --wordlist FILE [--mask lower --max-len 5]",
        );
        return;
    };

    // Read as bytes: leaked wordlists (rockyou) carry non-UTF-8 lines. A line
    // that is not valid UTF-8 cannot be a plaintext we care about, so it is
    // counted and skipped rather than poisoning the whole read.
    let mut non_utf8_lines: u64 = 0;
    for path in &wordlists {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                report::failure("md5_crack", &format!("cannot read {path}: {e}"), "check the wordlist path");
                return;
            }
        };
        for raw in bytes.split(|b| *b == b'\n') {
            let line = match std::str::from_utf8(raw) {
                Ok(s) => s.trim_end_matches('\r').trim(),
                Err(_) => {
                    non_utf8_lines += 1;
                    continue;
                }
            };
            if !line.is_empty() && !line.starts_with('#') {
                candidates.push(line.to_string());
            }
        }
    }

    if candidates.is_empty() && mask.is_none() {
        report::failure(
            "md5_crack",
            "no candidates: pass --wordlist FILE, --words 'a,b', or --mask",
            "give at least one candidate source",
        );
        return;
    }

    let mut data = run(&hash, &candidates, mask.as_deref(), max_len);
    if let Some(obj) = data.as_object_mut() {
        obj.insert("non_utf8_lines_skipped".to_string(), json!(non_utf8_lines));
    }
    if data["cracked"] == json!(true) {
        report::success("md5_crack", data, "use the cracked password to authenticate as the owner").expect("report");
    } else {
        report::success(
            "md5_crack",
            data,
            "no hit - widen the wordlist or raise --max-len with a mask",
        )
        .expect("report");
    }
}
