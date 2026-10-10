#!/usr/bin/env rust-script
//! name: raw_matrix
//! description: Byte-level request matrix - send N caller-controlled raw HTTP/1.1 requests (each on its own TLS/TCP connection) from one inline/file spec and report per-variant status, bytes, digest, Set-Cookie names, X-Cache/Age/Location, a marker hit and a body snippet, so a whole parsing/cache hypothesis space is provable in ONE envelope. Host note (kimi G1, cross-problem 2): the spec carries its own Host header per variant - do not also write Host in the request headers of the template or every variant 400s. Deadline note (kimi): no internal cap exists - budget variants x ~15s wall-clock yourself, or the run self-harms the window. [note](kimi 129 实证):编码/过滤面盘走两段序——先标签面盘(变体=标签集,锚定放行标签),再属性面盘(变体=事件属性集,锚定放行属性);两轮各一次矩阵,不要逐变体单发。 [sweep](W2 件化,129 实证 135+115 变体曾全手搓):模板的 line/headers/body 留 {{V}} 槽,--values 给值表(内联逗号表或 @file 每行一值,# 注释行跳过),模板×值表叉乘展开成整张矩阵,单调用覆盖整个面盘。
//! version: 1.1.4
//! args: <spec.json|@file|inline-json> [--quiet] [--values <a,b,c|@wordlist>]
//! keywords: raw, http, matrix, request-line, host-header, cache, parsing, ssrf, knife, 多变体一次发送对比
//!
//! ```cargo
//! [dependencies]
//! rustls = "0.23"
//! webpki-roots = "0.26"
//! serde_json = "1"
//! ```

use pi_rust_lib::serde_json::{json, Value};
use pi_rust_lib::timeout;
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        let tpl = json!({"name": "t", "line": "GET /?x={{V}} HTTP/1.1", "headers": ["X-T: {{V}}"], "body": "p={{V}}"});
        let e = expand_sweep(&tpl, "<xss>");
        let ok = e["line"].as_str().unwrap().contains("<xss>")
            && e["headers"][0].as_str().unwrap() == "X-T: <xss>"
            && e["body"].as_str().unwrap() == "p=<xss>"
            && e["name"].as_str().unwrap() == "t:<xss>";
        let bare = expand_sweep(&json!({"line": "GET /?{{V}}"}), "onresize");
        let ok = ok && bare["name"].as_str().unwrap() == "onresize" && !expand_sweep(&json!({"line": "GET /"}), "v").to_string().contains("{{V}}");
        pi_rust_lib::report::success(
            "raw_matrix",
            json!({"selftest": if ok { "ok" } else { "fail" }}),
            "sweep expansion covers line/headers/body/name",
        )
        .unwrap_or(());
        std::process::exit(if ok { 0 } else { 1 });
    }
    let mut spec_arg: Option<String> = None;
    let mut quiet = false;
    let mut values_arg: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quiet" => quiet = true,
            "--values" => {
                i += 1;
                if i < args.len() {
                    values_arg = Some(args[i].clone());
                }
            }
            other if !other.starts_with("--") && spec_arg.is_none() => spec_arg = Some(other.to_string()),
            _ => {}
        }
        i += 1;
    }
    let Some(spec_arg) = spec_arg else {
        pi_rust_lib::report::failure("raw_matrix", "missing spec", "call as: raw_matrix '<json>' | @file.json");
        std::process::exit(2);
    };
    let text = if let Some(path) = spec_arg.strip_prefix('@') {
        match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                pi_rust_lib::report::failure("raw_matrix", &format!("read {path}: {e}"), "pass a readable @spec file");
                std::process::exit(2);
            }
        }
    } else {
        spec_arg.clone()
    };
    let spec: Value = match pi_rust_lib::serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            pi_rust_lib::report::failure("raw_matrix", &format!("spec parse: {e}"), "spec is JSON: {host, variants:[{name,line,headers,body}]}");
            std::process::exit(2);
        }
    };
    let host = spec.get("host").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let port = spec.get("port").and_then(|v| v.as_u64()).unwrap_or(443) as u16;
    let tls = spec.get("tls").and_then(|v| v.as_bool()).unwrap_or(true);
    let marker = spec.get("marker").and_then(|v| v.as_str()).map(|s| s.to_string());
    let snippet = spec.get("snippet").and_then(|v| v.as_u64()).unwrap_or(160) as usize;
    if host.is_empty() {
        pi_rust_lib::report::failure("raw_matrix", "spec.host missing", "set spec.host to the TLS/TCP target hostname");
        std::process::exit(2);
    }
    let variants = spec.get("variants").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    // 面盘展开(W2):模板×值表叉乘,{{V}} 槽填值;无 --values 则原样单发。
    let variants: Vec<Value> = if let Some(va) = &values_arg {
        let values: Vec<String> = if let Some(path) = va.strip_prefix('@') {
            match std::fs::read_to_string(path) {
                Ok(t) => t
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .collect(),
                Err(e) => {
                    pi_rust_lib::report::failure("raw_matrix", &format!("read {path}: {e}"), "pass a readable wordlist file");
                    std::process::exit(2);
                }
            }
        } else {
            va.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        };
        if values.is_empty() {
            pi_rust_lib::report::failure("raw_matrix", "values list empty", "give inline a,b,c or @file (one per line, # comments skipped)");
            std::process::exit(2);
        }
        let mut out: Vec<Value> = Vec::new();
        for tpl in &variants {
            for val in &values {
                out.push(expand_sweep(tpl, val));
            }
        }
        out
    } else {
        variants
    };
    if variants.is_empty() {
        pi_rust_lib::report::failure("raw_matrix", "spec.variants empty", "add variants: [{name, line, headers:[..], body}]");
        std::process::exit(2);
    }

    let sub = |s: &str| s.replace("{{HOST}}", &host);
    let mut rows: Vec<Value> = Vec::new();
    for (idx, v) in variants.iter().enumerate() {
        timeout::check();
        let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let line = v
            .get("line")
            .and_then(|x| x.as_str())
            .map(|s| sub(s))
            .unwrap_or_else(|| "GET / HTTP/1.1".to_string());
        let headers: Vec<(String, String)> = v
            .get("headers")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|h| h.as_str())
                    .filter_map(|h| h.split_once(':'))
                    .map(|(k, val)| (k.trim().to_string(), sub(val.trim())))
                    .collect()
            })
            .unwrap_or_default();
        let body = v.get("body").and_then(|x| x.as_str()).map(|s| s.to_string());
        match send(&host, port, tls, &line, &headers, body.as_deref()) {
            Ok((status, status_text, head, resp_body, cookies)) => {
                let digest = digest_of(&resp_body);
                let hdr = |k: &str| -> Option<String> {
                    head.lines()
                        .skip(1)
                        .find(|l| l.to_ascii_lowercase().starts_with(&format!("{}:", k.to_lowercase())))
                        .and_then(|l| l.split_once(':'))
                        .map(|(_, val)| val.trim().to_string())
                };
                let hit = marker.as_ref().map(|m| resp_body.contains(m.as_str())).unwrap_or(false);
                let mut row = json!({
                    "i": idx,
                    "name": name,
                    "status": status,
                    "status_text": status_text,
                    "bytes": resp_body.len(),
                    "digest": format!("{digest:016x}"),
                    "cookies": cookies,
                    "x_cache": hdr("x-cache"),
                    "age": hdr("age"),
                    "location": hdr("location"),
                    "cache_control": hdr("cache-control"),
                });
                if marker.is_some() {
                    row["marker_hit"] = json!(hit);
                }
                if !quiet {
                    row["snippet"] = json!(resp_body.chars().take(snippet).collect::<String>());
                }
                rows.push(row);
            }
            Err(e) => rows.push(json!({ "i": idx, "name": name, "error": e })),
        }
    }

    let summary: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "name": r.get("name"),
                "status": r.get("status"),
                "bytes": r.get("bytes"),
                "digest": r.get("digest"),
                "marker_hit": r.get("marker_hit"),
            })
        })
        .collect();
    let data = json!({
        "host": host,
        "variants": rows.len(),
        "summary": summary,
        "rows": rows,
    });
    pi_rust_lib::report::success("raw_matrix", data, "compare per-variant status/bytes/digest to find the shape that diverges").expect("report success");
}

fn send(
    host: &str,
    port: u16,
    tls: bool,
    request_line: &str,
    headers: &[(String, String)],
    body: Option<&str>,
) -> Result<(u16, String, String, String, Vec<String>), String> {
    let mut request = format!("{request_line}\r\n");
    let has_host = headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("host"));
    if !has_host {
        request.push_str(&format!("Host: {host}\r\n"));
    }
    for (k, v) in headers {
        request.push_str(&format!("{k}: {v}\r\n"));
    }
    request.push_str("Connection: close\r\n\r\n");
    if let Some(b) = body {
        request.push_str(b);
    }

    let tcp = TcpStream::connect((host, port)).map_err(|e| format!("connect {host}:{port}: {e}"))?;
    let _ = tcp.set_read_timeout(Some(std::time::Duration::from_secs(15)));
    let mut raw = Vec::new();
    if tls {
        let mut roots = rustls::RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        let server_name = rustls::pki_types::ServerName::try_from(host.to_string()).map_err(|e| format!("bad server name: {e}"))?;
        let conn = rustls::ClientConnection::new(Arc::new(config), server_name).map_err(|e| format!("tls init: {e}"))?;
        let mut stream = rustls::StreamOwned::new(conn, tcp);
        stream.write_all(request.as_bytes()).map_err(|e| format!("write: {e}"))?;
        stream.read_to_end(&mut raw).map_err(|e| format!("read: {e}"))?;
    } else {
        let mut stream = tcp;
        stream.write_all(request.as_bytes()).map_err(|e| format!("write: {e}"))?;
        stream.read_to_end(&mut raw).map_err(|e| format!("read: {e}"))?;
    }

    let text = String::from_utf8_lossy(&raw).into_owned();
    let (head, resp_body) = match text.split_once("\r\n\r\n") {
        Some((h, b)) => (h.to_string(), b.to_string()),
        None => (text.clone(), String::new()),
    };
    let status_line = head.lines().next().unwrap_or("").to_string();
    let mut parts = status_line.split_whitespace();
    let _http = parts.next();
    let status: u16 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let status_text = parts.collect::<Vec<_>>().join(" ");
    let cookies: Vec<String> = head
        .lines()
        .skip(1)
        .filter(|l| l.to_ascii_lowercase().starts_with("set-cookie:"))
        .filter_map(|l| l.split_once(':'))
        .filter_map(|(_, v)| v.trim().split('=').next().map(|n| n.to_string()))
        .collect();
    Ok((status, status_text, head, resp_body, cookies))
}

/// 把模板变体的 {{V}} 槽填成 sweep 值(line/headers/body 三面),行名带值可辨。
fn expand_sweep(tpl: &Value, val: &str) -> Value {
    let mut out = tpl.clone();
    let sub = |s: &str| s.replace("{{V}}", val);
    if let Some(line) = out.get("line").and_then(|x| x.as_str()).map(|s| sub(s)) {
        out["line"] = Value::String(line);
    }
    if let Some(hdrs) = out.get("headers").and_then(|x| x.as_array()).cloned() {
        out["headers"] = Value::Array(
            hdrs
                .iter()
                .map(|h| h.as_str().map(|s| Value::String(sub(s))).unwrap_or_else(|| h.clone()))
                .collect(),
        );
    }
    if let Some(body) = out.get("body").and_then(|x| x.as_str()).map(|s| sub(s)) {
        out["body"] = Value::String(body);
    }
    let tpl_name = tpl.get("name").and_then(|x| x.as_str()).unwrap_or("");
    out["name"] = Value::String(if tpl_name.is_empty() {
        val.to_string()
    } else {
        format!("{tpl_name}:{val}")
    });
    out
}

fn digest_of(body: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    body.hash(&mut hasher);
    hasher.finish()
}
