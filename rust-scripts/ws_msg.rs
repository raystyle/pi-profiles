#!/usr/bin/env rust-script
//! name: ws_msg
//! description: Raw WebSocket handshake with caller-controlled extra headers - connect a ws/wss endpoint carrying the jar's session cookie and arbitrary injected handshake headers (X-Forwarded-For, Origin, X-Forwarded-Proto...), send one or more plain-text frames, then dump the handshake status/headers and every response frame, so handshake-manipulation (IP-ban bypass, origin bypass, header-driven routing) is provable in one envelope.
//! version: 1.0.0
//! args: <ws-url> --msg TEXT... [--header 'K: V']... [--jar PATH] [--timeout-ms N] [--read-ms N]
//! keywords: websocket, ws, wss, handshake, x-forwarded-for, chat, hunter, web-security
//!
//! ```cargo
//! [dependencies]
//! tungstenite = { version = "0.24", features = ["rustls-tls-webpki-roots"] }
//! rustls = { version = "0.23", features = ["ring"] }
//! url = "2"
//! ```

use pi_rust_lib::serde_json::{self, json, Value};
use std::collections::BTreeMap;
use tungstenite::client::IntoClientRequest;
use tungstenite::protocol::Message;
use tungstenite::stream::MaybeTlsStream;
use url::Url;

type Jar = BTreeMap<String, BTreeMap<String, String>>;

fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let mut url = String::new();
    let mut msgs: Vec<String> = Vec::new();
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut jar_path = format!("{home}/.pi-rs/agent/lab-jar.json");
    let mut timeout_ms: u64 = 15000;
    let mut read_ms: u64 = 4000;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--msg" => {
                i += 1;
                if i < args.len() {
                    msgs.push(args[i].clone());
                }
            }
            "--header" => {
                i += 1;
                if i < args.len() {
                    if let Some((k, v)) = args[i].split_once(':') {
                        headers.push((k.trim().to_string(), v.trim().to_string()));
                    }
                }
            }
            "--jar" => {
                i += 1;
                if i < args.len() {
                    jar_path = args[i].clone();
                }
            }
            "--timeout-ms" => {
                i += 1;
                if i < args.len() {
                    timeout_ms = args[i].parse().unwrap_or(timeout_ms);
                }
            }
            "--read-ms" => {
                i += 1;
                if i < args.len() {
                    read_ms = args[i].parse().unwrap_or(read_ms);
                }
            }
            other if !other.starts_with("--") && url.is_empty() => url = other.to_string(),
            other if !other.starts_with("--") => msgs.push(other.to_string()),
            _ => {}
        }
        i += 1;
    }
    if url.is_empty() || msgs.is_empty() {
        pi_rust_lib::report::failure(
            "ws_msg",
            "usage: ws_msg <ws-url> --msg TEXT... [--header 'K: V']... [--jar PATH]",
            "pass the websocket url and at least one message",
        );
        std::process::exit(2);
    }

    let parsed = Url::parse(&url).unwrap_or_else(|_| Url::parse("wss://invalid/").unwrap());
    let host = parsed.host_str().unwrap_or("").to_string();
    let cookie = std::fs::read_to_string(&jar_path)
        .ok()
        .and_then(|t| serde_json::from_str::<Jar>(&t).ok())
        .and_then(|jar| {
            jar.get(&host)
                .filter(|m| !m.is_empty())
                .map(|m| m.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("; "))
        });

    let mut request = match url.clone().into_client_request() {
        Ok(r) => r,
        Err(e) => {
            pi_rust_lib::report::failure("ws_msg", &format!("bad request: {e}"), "check the ws url");
            std::process::exit(1);
        }
    };
    if let Some(c) = &cookie {
        if let Ok(value) = c.parse() {
            request.headers_mut().insert("Cookie", value);
        }
    }
    for (k, v) in &headers {
        if let (Ok(name), Ok(value)) = (
            k.parse::<tungstenite::http::HeaderName>(),
            v.parse::<tungstenite::http::HeaderValue>(),
        ) {
            request.headers_mut().insert(name, value);
        }
    }

    let (mut socket, resp) = match tungstenite::connect(request) {
        Ok(v) => v,
        Err(e) => {
            pi_rust_lib::report::failure("ws_msg", &format!("connect failed: {e}"), "check the instance is up");
            std::process::exit(1);
        }
    };
    let handshake_status = resp.status().as_u16();
    let mut handshake_headers: Vec<String> = resp
        .headers()
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("<bin>")))
        .collect();
    handshake_headers.sort();

    let read_timeout = Some(std::time::Duration::from_millis(800.min(timeout_ms)));
    match socket.get_mut() {
        MaybeTlsStream::Plain(s) => {
            let _ = s.set_read_timeout(read_timeout);
        }
        MaybeTlsStream::Rustls(s) => {
            let _ = s.sock.set_read_timeout(read_timeout);
        }
        #[allow(unreachable_patterns)]
        _ => {}
    }

    for m in &msgs {
        if let Err(e) = socket.send(Message::text(m.clone())) {
            pi_rust_lib::report::failure("ws_msg", &format!("send failed: {e}"), "reconnect and retry");
            std::process::exit(1);
        }
    }

    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(read_ms);
    let mut frames: Vec<Value> = Vec::new();
    let mut count = 0usize;
    while std::time::Instant::now() < deadline && count < 200 {
        match socket.read() {
            Ok(Message::Text(text)) => {
                let s = text.as_str().to_string();
                frames.push(json!({ "type": "text", "data": s }));
                count += 1;
            }
            Ok(Message::Binary(b)) => {
                frames.push(json!({ "type": "binary", "len": b.len() }));
                count += 1;
            }
            Ok(Message::Ping(p)) => {
                let _ = socket.send(Message::Pong(p));
            }
            Ok(Message::Close(c)) => {
                frames.push(json!({ "type": "close", "data": format!("{:?}", c) }));
                break;
            }
            Ok(_) => {}
            Err(e) => {
                if let tungstenite::Error::Io(io) = &e {
                    use std::io::ErrorKind;
                    if matches!(io.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted) {
                        continue;
                    }
                }
                let msg = e.to_string();
                frames.push(json!({ "type": "error", "data": msg }));
                break;
            }
        }
    }
    let _ = socket.close(None);

    let data = json!({
        "url": url,
        "handshake_status": handshake_status,
        "handshake_headers": handshake_headers,
        "injected_headers": headers.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>(),
        "sent": msgs,
        "frames": frames,
    });
    pi_rust_lib::report::success("ws_msg", data, "read the frames: an 'Attack detected'/block banner proves the filter fired; retry with an injected X-Forwarded-For to bypass the IP ban").expect("report success");
}
