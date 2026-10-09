#!/usr/bin/env rust-script
//! name: h2_probe
//! description: Compile-and-run probe proving the h2 crate (tokio + rustls) resolves and can fetch a page over HTTP/2 with ALPN - the transport prerequisite for a multiplexed h2 brute driver.
//! version: 1.0.0
//! args: <url>
//! keywords: 漏洞猎手套件, h2, http2, probe, transport
//!
//! ```cargo
//! [dependencies]
//! tokio = { version = "1", features = ["rt-multi-thread", "net", "time", "macros", "io-util"] }
//! tokio-rustls = "0.26"
//! webpki-roots = "0.26"
//! h2 = "0.4"
//! http = "1"
//! url = "2"
//! ```

use pi_rust_lib::serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use url::Url;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let url = args.first().cloned().unwrap_or_default();
    if url.is_empty() {
        pi_rust_lib::report::failure("h2_probe", "missing url", "usage: h2_probe <url>");
        std::process::exit(2);
    }
    let parsed = Url::parse(&url).expect("url");
    let host = parsed.host_str().unwrap_or("").to_string();
    let port = parsed.port_or_known_default().unwrap_or(443);

    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let tls = Arc::new(
        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    );

    let tcp = TcpStream::connect((host.as_str(), port)).await.expect("tcp");
    tcp.set_nodelay(true).ok();
    let connector = TlsConnector::from(tls);
    let started = Instant::now();
    let tls_stream = connector
        .connect(ServerName::try_from(host.clone()).expect("sni"), tcp)
        .await
        .expect("tls");
    let tls_ms = started.elapsed().as_millis() as u64;

    let (send, conn) = h2::client::handshake(tls_stream).await.expect("h2 handshake");
    tokio::spawn(async move {
        let _ = conn.await;
    });
    let mut send = send.ready().await.expect("ready");

    let path = if parsed.path().is_empty() { "/" } else { parsed.path() };
    let req = http::Request::builder()
        .method("GET")
        .uri(format!("https://{host}{path}"))
        .header("user-agent", "h2-probe")
        .body(())
        .expect("request");
    let t0 = Instant::now();
    let (resp, _) = send.send_request(req, true).expect("send");
    let response = resp.await.expect("response");
    let status = response.status().as_u16();
    let body = read_body(response).await;
    let req_ms = t0.elapsed().as_millis() as u64;

    // second request on the same connection proves keep-alive
    let req2 = http::Request::builder()
        .method("GET")
        .uri(format!("https://{host}{path}"))
        .body(())
        .expect("request2");
    let t1 = Instant::now();
    let (resp2, _) = send.send_request(req2, true).expect("send2");
    let response2 = resp2.await.expect("response2");
    let status2 = response2.status().as_u16();
    let _ = read_body(response2).await;
    let req2_ms = t1.elapsed().as_millis() as u64;

    let ok = status == 200 && status2 == 200;
    let data = json!({
        "url": url,
        "tls_ms": tls_ms,
        "first_request_ms": req_ms,
        "second_request_ms": req2_ms,
        "status": status,
        "status2": status2,
        "body_len": body.len(),
        "crate": "h2 0.4 + tokio-rustls 0.26",
    });
    if ok {
        pi_rust_lib::report::success("h2_probe", data, "h2 transport available; reuse per-connection keep-alive for bulk requests")
            .expect("report");
    } else {
        pi_rust_lib::report::failure("h2_probe", "unexpected statuses or no keep-alive win", "inspect data");
        std::process::exit(1);
    }
}

async fn read_body(resp: http::Response<h2::RecvStream>) -> Vec<u8> {
    let mut body = resp.into_body();
    let mut out: Vec<u8> = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(20), async {
        while let Some(chunk) = body.data().await {
            match chunk {
                Ok(bytes) => {
                    let _ = body.flow_control().release_capacity(bytes.len());
                    out.extend_from_slice(&bytes);
                }
                Err(_) => break,
            }
        }
        let _ = body.trailers().await;
    })
    .await;
    out
}
