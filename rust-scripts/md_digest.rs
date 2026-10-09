#!/usr/bin/env rust-script
//! name: md_digest
//! description: Markdown 语料摘要件 - 对一个目录下的 *.md 逐文件取 H1 标题、frontmatter link 目标与指定的 H2 节(按每节字符预算截断,可选 --bullets 限条数),渲染成可读摘要文本落盘并回一份索引信封,供大体量实录语料按「方法节优先」批量速览(全量蒸馏批的扫描臂)。
//! version: 1.0.0
//! args: <dir> [--sections 'A:300|B:150'] [--files SUBSTR] [--exclude SUBSTR] [--bullets N] [--max-files N] [--out FILE] [--selftest]
//! keywords: markdown, digest, sections, corpus, survey, distillation, records
//!
//! ```cargo
//! ```
use pi_rust_lib::serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

struct Spec {
    name: String,
    budget: usize,
}

fn parse_sections(s: &str) -> Vec<Spec> {
    let mut out = Vec::new();
    for part in s.split('|') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (name, budget) = match part.rsplit_once(':') {
            Some((n, b)) => (n.trim().to_string(), b.trim().parse::<usize>().unwrap_or(300)),
            None => (part.to_string(), 300usize),
        };
        out.push(Spec { name, budget });
    }
    out
}

fn truncate_chars(s: &str, budget: usize) -> (String, bool) {
    let mut out = String::new();
    let mut n = 0usize;
    for ch in s.chars() {
        if n >= budget {
            return (out, true);
        }
        out.push(ch);
        n += 1;
    }
    (out, false)
}

/// Split the file into (h1, link targets, h2 name -> body lines).
fn parse_file(text: &str) -> (String, Vec<String>, Vec<(String, Vec<String>)>) {
    let mut h1 = String::new();
    let mut links: Vec<String> = Vec::new();
    let mut sections: Vec<(String, Vec<String>)> = Vec::new();
    let mut cur: Option<usize> = None;
    let mut in_fm = false;
    let mut fm_done = false;
    let mut hi = 0usize;

    let lines: Vec<&str> = text.lines().collect();
    while hi < lines.len() {
        let raw = lines[hi].trim_end_matches('\r');
        hi += 1;
        if hi == 1 && raw.trim() == "---" {
            in_fm = true;
            continue;
        }
        if in_fm {
            if raw.trim() == "---" {
                in_fm = false;
                fm_done = true;
                continue;
            }
            if raw.trim_start().starts_with("- target:") {
                let t = raw.trim_start().trim_start_matches("- target:").trim();
                links.push(t.trim_matches('"').to_string());
            }
            continue;
        }
        let _ = fm_done;
        if let Some(rest) = raw.strip_prefix("## ") {
            let name = rest.trim().to_string();
            sections.push((name, Vec::new()));
            cur = Some(sections.len() - 1);
            continue;
        }
        if let Some(rest) = raw.strip_prefix("# ") {
            if h1.is_empty() {
                h1 = rest.trim().to_string();
            }
            continue;
        }
        if let Some(i) = cur {
            sections[i].1.push(raw.to_string());
        }
    }
    (h1, links, sections)
}

/// Render a section body: drop blank lines, keep up to `bullets` bullet lines,
/// collapse the rest into flowing text, truncate to budget.
fn render(body: &[String], budget: usize, bullets: Option<usize>) -> String {
    let mut kept: Vec<String> = Vec::new();
    let mut bullet_count = 0usize;
    for line in body {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let is_bullet = t.starts_with("- ") || t.starts_with("* ");
        if is_bullet {
            bullet_count += 1;
            if let Some(max) = bullets {
                if bullet_count > max {
                    continue;
                }
            }
            kept.push(t.to_string());
        } else {
            kept.push(t.to_string());
        }
    }
    let joined = kept.join(" ");
    let (mut text, cut) = truncate_chars(&joined, budget);
    if cut {
        text.push('…');
    }
    text
}

fn collect_md(dir: &Path, out: &mut Vec<PathBuf>) {
    let rd = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_md(&p, out);
        } else if p.extension().map(|x| x == "md").unwrap_or(false) {
            out.push(p);
        }
    }
}

fn run(args: &[String]) -> Result<Value, String> {
    let dir = args
        .first()
        .cloned()
        .ok_or_else(|| "usage: md_digest <dir> [--sections 'A:300|B:150'] [--out FILE]".to_string())?;
    let mut sections_spec = String::from("题面分析:200|陷阱与注意事项:260");
    let mut files_filter: Option<String> = None;
    let mut exclude: Option<String> = None;
    let mut bullets: Option<usize> = None;
    let mut max_files: Option<usize> = None;
    let mut out_path = String::from("/tmp/md_digest.txt");

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--sections" => {
                i += 1;
                sections_spec = args.get(i).cloned().unwrap_or_default();
            }
            "--files" => {
                i += 1;
                files_filter = args.get(i).cloned();
            }
            "--exclude" => {
                i += 1;
                exclude = args.get(i).cloned();
            }
            "--bullets" => {
                i += 1;
                bullets = args.get(i).and_then(|v| v.parse().ok());
            }
            "--max-files" => {
                i += 1;
                max_files = args.get(i).and_then(|v| v.parse().ok());
            }
            "--out" => {
                i += 1;
                out_path = args.get(i).cloned().unwrap_or(out_path);
            }
            other => return Err(format!("unknown flag {other}")),
        }
        i += 1;
    }

    let specs = parse_sections(&sections_spec);
    let mut files: Vec<PathBuf> = Vec::new();
    collect_md(Path::new(&dir), &mut files);
    files.sort();

    let mut body = String::new();
    let mut index: Vec<Value> = Vec::new();
    let mut used = 0usize;
    for p in &files {
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
        if name.starts_with('_') {
            continue;
        }
        if let Some(f) = &files_filter {
            if !name.contains(f.as_str()) {
                continue;
            }
        }
        if let Some(x) = &exclude {
            if name.contains(x.as_str()) {
                continue;
            }
        }
        if let Some(m) = max_files {
            if used >= m {
                break;
            }
        }
        let text = match fs::read_to_string(p) {
            Ok(t) => t,
            Err(e) => {
                body.push_str(&format!("### {name} | unreadable: {e}\n"));
                continue;
            }
        };
        let (h1, links, secs) = parse_file(&text);
        used += 1;
        body.push_str(&format!("### {name} | {h1}\n"));
        if !links.is_empty() {
            body.push_str(&format!("links: {}\n", links.join(", ")));
        }
        let mut found: Vec<Value> = Vec::new();
        for spec in &specs {
            let hit = secs.iter().find(|(n, _)| n == &spec.name);
            match hit {
                Some((_, b)) => {
                    let r = render(b, spec.budget, bullets);
                    body.push_str(&format!("{}: {r}\n", spec.name));
                    found.push(json!({"name": spec.name, "chars": r.chars().count()}));
                }
                None => {}
            }
        }
        body.push('\n');
        index.push(json!({"file": name, "title": h1, "sections": found}));
    }

    if let Some(parent) = Path::new(&out_path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&out_path, &body).map_err(|e| format!("writing {out_path}: {e}"))?;

    Ok(json!({
        "dir": dir,
        "out": out_path,
        "files_digested": used,
        "bytes": body.len(),
        "sections": specs.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
        "index": index,
    }))
}

fn selftest() -> Result<Value, String> {
    let sample = "---\nlinks:\n  - target: a/b\n    relation: supports\n---\n\n# 07 · T\n\n## 题面分析\n\n甲句。\n乙句。\n\n## 陷阱与注意事项\n\n- 坑一\n- 坑二\n- 坑三\n";
    let (h1, links, secs) = parse_file(sample);
    let ok_title = h1 == "07 · T";
    let ok_links = links == vec!["a/b".to_string()];
    let ok_secs = secs.len() == 2 && secs[0].0 == "题面分析";
    let r1 = render(&secs[0].1, 4, None);
    let r2 = render(&secs[1].1, 200, Some(2));
    let ok_trunc = r1.starts_with("甲句。") && r1.ends_with('…') && r1.chars().count() == 5;
    let ok_bullets = r2 == "- 坑一 - 坑二";
    let all = ok_title && ok_links && ok_secs && ok_trunc && ok_bullets;
    Ok(json!({
        "title": ok_title, "links": ok_links, "sections": ok_secs,
        "truncate": ok_trunc, "bullets": ok_bullets, "render_sample": r2, "pass": all,
    }))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        match selftest() {
            Ok(v) => {
                let ok = v["pass"] == json!(true);
                let _ = pi_rust_lib::report::success("md_digest_selftest", v, "if pass=false fix the parser");
                if !ok {
                    std::process::exit(1);
                }
            }
            Err(e) => pi_rust_lib::report::failure("md_digest_selftest", &e, "check the selftest sample"),
        }
        return;
    }
    match run(&args) {
        Ok(v) => {
            let _ = pi_rust_lib::report::success("md_digest", v, "read the --out file; narrow with --files/--sections for detail");
        }
        Err(e) => pi_rust_lib::report::failure("md_digest", &e, "md_digest <dir> [--sections 'A:300|B:150'] [--out FILE]"),
    }
}
