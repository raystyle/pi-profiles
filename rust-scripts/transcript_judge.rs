#!/usr/bin/env rust-script
//! name: transcript_judge
//! description: pi session-transcript judgement-surface probe - per line it classifies content block types (text/thinking/toolCall) and which marker (is-solved/Congratulations/终态...) lands in which block, so the terminal-anchor line (assistant text) is separable from a thinking-block template parrot (false positive); reports each anchor's distance from EOF for tail-window sizing, and flags whether a line carries "type":"text" and/or a "thinking" marker in its raw bytes.
//! version: 1.0.0
//! args: <transcript.jsonl> [--markers 'a,b,c'] [--tail N]
//! keywords: 自省台, transcript, judgement, 判定面, anchor, 真锚, 鹦鹉, thinking, 尾窗, tail
//!
//! ```cargo
//! ```
use pi_rust_lib::serde_json::{self, json, Value};

fn snip(s: &str, n: usize) -> String {
    let mut out: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        out.push('…');
    }
    out.replace('\n', "⏎")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    let mut path = String::new();
    let mut markers: Vec<String> = vec![
        "is-solved".to_string(),
        "Congratulations".to_string(),
        "终态".to_string(),
    ];
    let mut tail = 8usize;
    let mut judge = false;
    let mut lab = String::new();
    let mut arm = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--markers" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    markers = v.split(',').map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();
                }
            }
            "--tail" => {
                i += 1;
                tail = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(8);
            }
            "--judge" => {
                judge = true;
                if let Some(l) = args.get(i + 1) {
                    lab = l.clone();
                    i += 1;
                }
                if let Some(a) = args.get(i + 1) {
                    arm = a.clone();
                    i += 1;
                }
            }
            _ => {
                if path.is_empty() {
                    path = args[i].clone();
                }
            }
        }
        i += 1;
    }
    if path.is_empty() {
        pi_rust_lib::report::failure(
            "transcript_judge",
            "missing <transcript.jsonl>",
            "pass the pi session transcript path",
        );
        std::process::exit(2);
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            pi_rust_lib::report::failure(
                "transcript_judge",
                &format!("cannot read {path}: {e}"),
                "check the path",
            );
            std::process::exit(2);
        }
    };

    let total = content.lines().count();
    let mut matched: Vec<Value> = Vec::new();
    let mut assistant_text_lines: Vec<Value> = Vec::new();
    let mut text_only: Vec<usize> = Vec::new();
    let mut thinking_only: Vec<usize> = Vec::new();
    let mut mixed: Vec<usize> = Vec::new();
    let mut receipts: Vec<Value> = Vec::new();
    let mut anchors: Vec<Value> = Vec::new();

    for (idx, raw) in content.lines().enumerate() {
        let lineno = idx + 1;
        let obj: Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let msg = obj.get("message").and_then(|m| m.as_object());
        let role = msg
            .and_then(|m| m.get("role"))
            .and_then(|r| r.as_str())
            .unwrap_or("?")
            .to_string();
        let mut types: Vec<String> = Vec::new();
        let mut text_join = String::new();
        let mut thinking_join = String::new();
        if let Some(m) = msg {
            if let Some(arr) = m.get("content").and_then(|c| c.as_array()) {
                for c in arr {
                    let t = c.get("type").and_then(|x| x.as_str()).unwrap_or("?");
                    types.push(t.to_string());
                    if t == "text" {
                        if let Some(s) = c.get("text").and_then(|x| x.as_str()) {
                            text_join.push_str(s);
                            text_join.push('\n');
                        }
                    } else if t == "thinking" {
                        if let Some(s) = c.get("thinking").and_then(|x| x.as_str()) {
                            thinking_join.push_str(s);
                            thinking_join.push('\n');
                        }
                    }
                }
            }
        }
        let has_text = types.iter().any(|t| t == "text");
        let has_thinking = types.iter().any(|t| t == "thinking");
        if role == "assistant" {
            if has_text && !has_thinking {
                text_only.push(lineno);
            } else if has_thinking && !has_text {
                thinking_only.push(lineno);
            } else if has_text && has_thinking {
                mixed.push(lineno);
            }
            if has_text {
                let hits: Vec<&String> = markers.iter().filter(|m| text_join.contains(m.as_str())).collect();
                assistant_text_lines.push(json!({
                    "line": lineno,
                    "dist_from_eof": total - lineno,
                    "has_thinking": has_thinking,
                    "text_marker_hits": hits,
                    "text_snippet": snip(&text_join, 140),
                }));
            }
        }
        if judge {
            // structured receipts: toolResult text carrying a solved:true/false verdict
            if role == "toolResult" && (text_join.contains("\"solved\":true") || text_join.contains("solved:true") || text_join.contains("\"solved\":false") || text_join.contains("solved:false")) {
                let ok = text_join.contains("\"solved\":true") || text_join.contains("solved:true");
                receipts.push(json!({"line": lineno, "solved": ok, "snippet": snip(&text_join, 120)}));
            }
            // terminal anchors: assistant TEXT block carrying 终态(<lab>/<arm>): not in template form
            if role == "assistant" && !lab.is_empty() {
                let tag = format!("终态({}/{})", lab, arm);
                if let Some(pos) = text_join.find(&tag) {
                    let after: String = text_join[pos + tag.len()..].trim_start_matches(|c| c == ' ' || c == '\u{3000}').to_string();
                    let is_template = after.starts_with('。') || after.starts_with("<one sentence") || after.is_empty();
                    if !is_template {
                        anchors.push(json!({"line": lineno, "dist_from_eof": total - lineno, "tail": snip(&after, 120)}));
                    }
                }
            }
        }
        let raw_hit: Vec<&String> = markers.iter().filter(|m| raw.contains(m.as_str())).collect();
        if raw_hit.is_empty() {
            continue;
        }
        let text_hits: Vec<&String> = markers.iter().filter(|m| text_join.contains(m.as_str())).collect();
        let think_hits: Vec<&String> = markers.iter().filter(|m| thinking_join.contains(m.as_str())).collect();
        matched.push(json!({
            "line": lineno,
            "role": role,
            "types": types,
            "has_text": has_text,
            "has_thinking": has_thinking,
            "raw_has_type_text": raw.contains("\"type\":\"text\""),
            "raw_has_thinking_token": raw.contains("\"thinking\""),
            "marker_in_text": text_hits,
            "marker_in_thinking": think_hits,
            "marker_in_raw_other": raw_hit.len() as i64
                - text_hits.len() as i64 - think_hits.len() as i64,
            "text_snippet": snip(&text_join, 160),
            "thinking_snippet": snip(&thinking_join, 200),
        }));
    }

    let data = json!({
        "file": path,
        "lines": total,
        "markers": markers,
        "tail_window": tail,
        "matched_lines": matched,
        "assistant_text_lines": assistant_text_lines,
        "text_only_lines": text_only,
        "thinking_only_lines": thinking_only,
        "mixed_lines": mixed,
        "text_only_from_eof": text_only.iter().map(|l| total - l).collect::<Vec<usize>>(),
    });
    if judge {
        // verdict priority: structured receipt > assistant anchor vocab > none
        let mut solved = 0usize;
        let mut class = "none";
        let mut line = 0usize;
        let mut dist = usize::MAX;
        let mut tail_out = String::new();
        if let Some(last_true) = receipts.iter().filter(|r| r.get("solved") == Some(&json!(true))).next_back() {
            solved = 1; class = "receipt"; line = last_true["line"].as_u64().unwrap_or(0) as usize;
            dist = total.saturating_sub(line); tail_out = last_true["snippet"].as_str().unwrap_or("").to_string();
        } else if let Some(last_anchor) = anchors.last() {
            class = "anchor"; line = last_anchor["line"].as_u64().unwrap_or(0) as usize;
            dist = last_anchor["dist_from_eof"].as_u64().unwrap_or(0) as usize;
            tail_out = last_anchor["tail"].as_str().unwrap_or("").to_lowercase();
            let vocab = ["solved", "congratulations", "is-solved", "congrats", "已翻", "判对", "解出", "通过"];
            if vocab.iter().any(|v| tail_out.contains(v)) { solved = 1; }
        }
        let judge_out = json!({
            "solved": solved, "class": class, "line": line,
            "dist_from_eof": if dist == usize::MAX { Value::Null } else { json!(dist) },
            "anchor_tail": tail_out,
            "receipts_seen": receipts.len(), "anchors_seen": anchors.len(),
        });
        pi_rust_lib::report::success(
            "transcript_judge",
            judge_out,
            "judge mode: receipt>solved:true wins; else last assistant anchor scored by vocab",
        )
        .expect("report");
        return;
    }
    pi_rust_lib::report::success(
        "transcript_judge",
        data,
        "read matched_lines: an assistant line with raw_has_type_text=true and raw_has_thinking_token=false is the true anchor; a thinking-only line is the parrot",
    )
    .expect("report");
}

fn selftest() {
    let sample = r#"{"type":"message","message":{"role":"assistant","content":[{"type":"thinking","thinking":"banner shows is-solved (template parrot)"}]}}
{"type":"message","message":{"role":"toolResult","content":[{"type":"text","text":"<section class='is-solved'>"}]}}
{"type":"message","message":{"role":"assistant","content":[{"type":"text","text":"终态(lab-x/A): solved"}]}}"#;
    let tmp = std::env::temp_dir().join("transcript_judge_selftest.jsonl");
    std::fs::write(&tmp, sample).expect("write");
    let content = std::fs::read_to_string(&tmp).expect("read");
    let mut text_only = 0;
    let mut think_only = 0;
    let mut parrot_raw_thinking = false;
    let mut anchor_raw_type_text = false;
    for (idx, raw) in content.lines().enumerate() {
        let _ = idx;
        let obj: Value = serde_json::from_str(raw).unwrap();
        let m = obj.get("message").unwrap().as_object().unwrap();
        let role = m.get("role").unwrap().as_str().unwrap();
        let mut ht = false;
        let mut hk = false;
        for c in m.get("content").unwrap().as_array().unwrap() {
            match c.get("type").unwrap().as_str().unwrap() {
                "text" => ht = true,
                "thinking" => hk = true,
                _ => {}
            }
        }
        if role == "assistant" && ht && !hk {
            text_only += 1;
            if raw.contains("\"type\":\"text\"") && !raw.contains("\"thinking\"") {
                anchor_raw_type_text = true;
            }
        }
        if role == "assistant" && hk && !ht {
            think_only += 1;
            if raw.contains("\"thinking\"") && !raw.contains("\"type\":\"text\"") {
                parrot_raw_thinking = true;
            }
        }
    }
    let ok = text_only == 1 && think_only == 1 && parrot_raw_thinking && anchor_raw_type_text;
    let data = json!({
        "selftest": if ok {"ok"} else {"fail"},
        "text_only": text_only,
        "think_only": think_only,
        "parrot_raw_thinking_not_text": parrot_raw_thinking,
        "anchor_raw_type_text_not_thinking": anchor_raw_type_text,
    });
    let _ = std::fs::remove_file(&tmp);
    if ok {
        pi_rust_lib::report::success("transcript_judge", data, "selftest passed; ready to run").expect("report");
    } else {
        pi_rust_lib::report::failure("transcript_judge", "selftest failed", "inspect block classification");
        std::process::exit(1);
    }
}
