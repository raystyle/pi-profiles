#!/usr/bin/env rust-script
//! name: chrome_cookies
//! description: Decrypt a Chrome/Chromium Linux cookie DB offline (v10 = AES-128-CBC, key PBKDF2("peanuts","saltysalt",1,16), legacy 16-byte prefix + PKCS7) and export a host->name->value jar that lab_http/objref_scan read natively.
//! version: 1.0.1
//! args: <cookie-db-path> [--host SUBSTR] [--out JAR] [--password SECRET]
//! keywords: chrome, chromium, cookies, decrypt, session, jar, v10, cbc, aes
//!
//! ```cargo
//! [dependencies]
//! rusqlite = { version = "0.31", features = ["bundled"] }
//! aes = "0.8"
//! cbc = { version = "0.1", features = ["alloc", "block-padding"] }
//! pbkdf2 = "0.12"
//! sha1 = "0.10"
//! ```

use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use pi_rust_lib::serde_json::{self, json, Map, Value};
use rusqlite::{Connection, OpenFlags};
use std::collections::BTreeMap;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Jar = BTreeMap<String, BTreeMap<String, String>>;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let mut db = String::new();
    let mut host_filter: Option<String> = None;
    let mut out = format!("{home}/.pi-rs/agent/chrome-jar.json");
    let mut password = "peanuts".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--host" => {
                i += 1;
                if i < args.len() {
                    host_filter = Some(args[i].to_lowercase());
                }
            }
            "--out" => {
                i += 1;
                if i < args.len() {
                    out = args[i].clone();
                }
            }
            "--password" => {
                i += 1;
                if i < args.len() {
                    password = args[i].clone();
                }
            }
            other if !other.starts_with("--") && db.is_empty() => db = other.to_string(),
            _ => {}
        }
        i += 1;
    }

    if db.is_empty() {
        pi_rust_lib::report::failure(
            "chrome_cookies",
            "missing cookie db path",
            "call as: chrome_cookies <profile>/Default/Cookies [--host portswigger.net] [--out jar.json]", // binding-exempt: usage example of --host filter
        );
        std::process::exit(2);
    }

    let key = derive_key(&password);
    let con = match Connection::open_with_flags(&db, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => {
            pi_rust_lib::report::failure(
                "chrome_cookies",
                &format!("cannot open {db}: {e}"),
                "check the path points at a Chrome Cookies sqlite file",
            );
            std::process::exit(1);
        }
    };

    let mut stmt = match con.prepare("select host_key, name, value, encrypted_value from cookies") {
        Ok(s) => s,
        Err(e) => {
            pi_rust_lib::report::failure("chrome_cookies", &format!("no cookies table: {e}"), "is this a Chrome Cookies db?");
            std::process::exit(1);
        }
    };
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<Vec<u8>>>(3)?,
        ))
    });

    let mut jar: Jar = BTreeMap::new();
    let mut hosts: Map<String, Value> = Map::new();
    let mut cookies = 0usize;
    let mut undecryptable = 0usize;
    let mut decryptable = 0usize;

    if let Ok(iter) = rows {
        for row in iter.flatten() {
            let (host_key, name, value, encrypted) = row;
            if let Some(filter) = &host_filter {
                if !host_key.to_lowercase().contains(filter.as_str()) {
                    continue;
                }
            }
            let plain = match value {
                Some(v) if !v.is_empty() => v,
                _ => match encrypted {
                    Some(ev) if ev.len() > 19 => match decrypt(&ev, &key) {
                        Some(text) => {
                            decryptable += 1;
                            text
                        }
                        None => {
                            undecryptable += 1;
                            continue;
                        }
                    },
                    _ => continue,
                },
            };
            let host = host_key.trim_start_matches('.').to_string();
            jar.entry(host.clone()).or_default().insert(name.clone(), plain);
            let entry = hosts.entry(host).or_insert_with(|| json!([]));
            if let Some(arr) = entry.as_array_mut() {
                arr.push(json!(name));
            }
            cookies += 1;
        }
    }

    if let Some(dir) = std::path::Path::new(&out).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&out, serde_json::to_string_pretty(&jar).unwrap_or_default()) {
        pi_rust_lib::report::failure("chrome_cookies", &format!("cannot write {out}: {e}"), "pick a writable --out path");
        std::process::exit(1);
    }

    let data = json!({
        "db": db,
        "out": out,
        "cookies": cookies,
        "decrypted": decryptable,
        "undecryptable": undecryptable,
        "hosts": Value::Object(hosts),
    });
    let next = if cookies == 0 {
        "no matching cookies; widen --host or check the profile path"
    } else if undecryptable > 0 {
        "some values failed to decrypt; the profile may not use the peanuts keyring-free store"
    } else {
        "pass the jar to lab_http/objref_scan with --jar"
    };
    pi_rust_lib::report::success("chrome_cookies", data, next).expect("report success");
}

/// PBKDF2-HMAC-SHA1 over the OSCrypt "basic" password; 16-byte key for AES-128.
fn derive_key(password: &str) -> [u8; 16] {
    let mut key = [0u8; 16];
    pbkdf2::pbkdf2_hmac::<sha1::Sha1>(password.as_bytes(), b"saltysalt", 1, &mut key);
    key
}

/// v10/v11 = 16-byte IV + AES-128-CBC ciphertext; the plaintext is a legacy
/// 16-byte prefix (dropped) followed by the value and PKCS7 padding.
fn decrypt(encrypted: &[u8], key: &[u8; 16]) -> Option<String> {
    let prefix = &encrypted[..3];
    if prefix != b"v10" && prefix != b"v11" {
        return None;
    }
    let iv = &encrypted[3..19];
    let ct = &encrypted[19..];
    if ct.is_empty() || ct.len() % 16 != 0 {
        return None;
    }
    let mut buf = ct.to_vec();
    let plain = Aes128CbcDec::new_from_slices(key, iv)
        .ok()?
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .ok()?;
    let body = if plain.len() > 16 { &plain[16..] } else { plain };
    Some(String::from_utf8_lossy(body).into_owned())
}
