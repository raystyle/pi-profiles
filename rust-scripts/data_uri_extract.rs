#!/usr/bin/env rust-script
//! name: data_uri_extract
//! description: 内联资源提取件 - 从 HTML/文本文件里取出内联 data: URI 载荷(可按周边文本 --contains 选定哪一个)并按 MIME 落盘成二进制文件,回报次序、mime、字节数与首尾字节,供 OCR 或视觉读取内联图片(captcha 等)。
//! version: 1.0.0
//! args: <file> --out FILE [--contains SUBSTR] [--index N]
//! keywords: data-uri, base64, inline-image, captcha, png, extract
//!
//! ```cargo
//! [dependencies]
//! base64 = "0.22"
//! ```

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use pi_rust_lib::serde_json::json;

const USAGE: &str = "data_uri_extract <file> --out FILE [--contains SUBSTR] [--index N]";

fn fail(msg: &str, next: &str) -> ! {
    pi_rust_lib::report::failure(
        "data_uri_extract",
        &format!("usage: {USAGE} :: {msg}"),
        next,
    );
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let val = |i: usize| args.get(i + 1).cloned().unwrap_or_default();
    let mut file = String::new();
    let mut out = String::new();
    let mut contains = String::new();
    let mut index: usize = 0;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => { out = val(i); i += 2; }
            "--contains" => { contains = val(i); i += 2; }
            "--index" => { index = val(i).parse().unwrap_or(0); i += 2; }
            a if a.starts_with("--") => fail(&format!("unknown flag {a}"), USAGE),
            a => { file = a.to_string(); i += 1; }
        }
    }
    if file.is_empty() || out.is_empty() {
        fail("need <file> and --out", USAGE);
    }

    let text = match std::fs::read(&file) {
        Ok(b) => String::from_utf8_lossy(&b).to_string(),
        Err(e) => fail(&format!("read {file}: {e}"), USAGE),
    };

    let mut found: Vec<(String, String, usize)> = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel) = text[cursor..].find(";base64,") {
        let at = cursor + rel;
        let payload_start = at + ";base64,".len();
        let mime = match text[..at].rfind("data:") {
            Some(m) => text[m + 5..at].to_string(),
            None => String::new(),
        };
        let rest = &text[payload_start..];
        let end = rest
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '+' && c != '/' && c != '=' && c != '-')
            .unwrap_or(rest.len());
        let payload = rest[..end].to_string();
        let ctx_start = text[..at].len().saturating_sub(300);
        let ctx_start = {
            let mut s = ctx_start;
            while s < text.len() && !text.is_char_boundary(s) { s += 1; }
            s
        };
        let ctx = text[ctx_start..at].to_string();
        if contains.is_empty() || ctx.contains(&contains) {
            found.push((mime, payload, at));
        }
        cursor = payload_start;
    }

    if found.is_empty() {
        fail(
            &format!("no data: URI matched (contains={contains:?})"),
            "fetch the page again and pass --contains with a nearby id/attribute value",
        );
    }
    let (mime, payload, at) = &found[index.min(found.len() - 1)];
    let bytes = match STANDARD.decode(payload) {
        Ok(b) => b,
        Err(e) => fail(&format!("base64 decode: {e}"), USAGE),
    };
    if let Err(e) = std::fs::write(&out, &bytes) {
        fail(&format!("write {out}: {e}"), USAGE);
    }

    let head: Vec<String> = bytes.iter().take(8).map(|b| format!("{b:02x}")).collect();
    let tail: Vec<String> = bytes.iter().rev().take(4).rev().map(|b| format!("{b:02x}")).collect();
    let data = json!({
        "candidates": found.len(),
        "chosen_index": index.min(found.len() - 1),
        "offset_in_file": at,
        "mime": mime,
        "payload_chars": payload.len(),
        "bytes": bytes.len(),
        "head_hex": head.join(" "),
        "tail_hex": tail.join(" "),
        "out": out,
    });
    pi_rust_lib::report::success(
        "data_uri_extract",
        data,
        "read the extracted image with the read tool (vision) or OCR it with doc_ocr",
    )
    .expect("report");
}
