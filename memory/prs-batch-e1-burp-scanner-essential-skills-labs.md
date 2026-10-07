---
metadata:
  node_type: memory
name: "PRS Batch E1 - burp-scanner essential-skills labs"
description: "Batch E1 (2 burp-scanner labs): 0/2 solved - lab1 surface mapped (stock endpoint = per-string pseudo-stock oracle, traversal/SQLi/cmdi/SSRF/XXE ruled out, net scan 53 probes 0 suspects), lab2 cookie structure cracked (username:token, split on FIRST colon, unknown user -> 500, name field escaped/64 chars) but no privileged renderer found; OOB base verified (oob.dthack.io DNS + :9999 HTTP on ubuntu@47.131.34.33); range_launch needs per-lab --widget-source"
last_updated: 2026-10-07T20:15:45+08:00
created: 2026-10-07T20:15:45+08:00
---

## Batch E1 (essential-skills / using-burp-scanner-during-manual-testing, 2 labs, both previously mis-flagged blocked-burp; no burp available)

Result: 0/2 solved. Lab 1 unresolved with a mapped-and-ruled-out surface; lab 2 structure mapped, exploitation not reached.

### Lab 1 - lab-discovering-vulnerabilities-quickly-with-targeted-scanning (lab id 096E7513..., 10-minute timer, read /etc/passwd)
- App = shop clone. Endpoints: `/`, `/product?productId=N`, `POST /product/stock`, `/image/<path>`, `/filter` (200, unlinked, looks like the listing; 11236 vs home 11138 bytes).
- `POST /product/stock` (form, exactly productId/storeId; Content-Type ignored - JSON body under `application/json` still parsed; XML/JSON bodies -> 400 "No such product or store"; GET -> 405) returns a **deterministic pseudo-stock integer 0-999 for ANY value**: `1`->732, `/etc/passwd`->898, `/dev/null`->217, `AAAAA`->941; same string gives the same number across instance relaunches. Equivalent file paths (`/etc/passwd` vs `/etc//passwd` vs `/etc/./passwd`) give different numbers => not filesystem semantics, just an opaque per-string value.
- Ruled out: path traversal (`/image/../..`,`..%2f`,`..%252f`, `%2e%2e%2f`, `/resources/..%2f..`, `?filename=` on /image all 404), SQLi (battery + manual `<q> SLEEP/pg_sleep/LOAD_FILE/boolean pairs`), time-based cmdi (`;sleep 8`, `| sleep 8`: no delay), SSRF (no hangs, no fetches to the canary), XXE (neither `application/xml` nor `text/xml` body is parsed), template injection (`{{7*7}}`, `${7*7}`, `#{7*7}`, `<%= %>`), reflected-XSS (productId is echoed only in a JSON 400 body), session-cookie traversal, Accept-Language traversal. 40-name endpoint wordlist: nothing beyond the above.
- net scan first live use: `dbg_cli net scan --pattern '*/product/stock' --battery sqli,xss,cmdi,path` captured a template with slots cookie:session / form:productId / form:storeId / header:Referer / header:User-Agent -> **53 probes, 21 cookie-unsafe skipped, suspects EMPTY**. `--battery` accepts only the subset sqli,xss,cmdi,path (`xxe,ssrf,ssti,deser` -> "net: empty battery"), so an XXE/SSRF-family lab is outside the battery's coverage. net scan needs the templated request to fire while it waits (arm in background, then click), and refuses a pattern already armed by `net intercept`.
- Timer/launch: 10 min -> page becomes "Time's up!" with `GET /try-again` (302 -> `/`) to restart; repeated range_launch returns the existing instance. App was reached first via `GET /product` -> "Click stock" -> browser click -> desk capture.

### Lab 2 - lab-scanning-non-standard-data-structures (lab id AA337200..., wiener:peter, delete carlos)
- App = blog clone: `/`, `/post?postId=N`, `POST /post/comment`, `/login` (csrf hidden field), `/my-account`, `/my-account/change-email`, `/admin`, `/logout`.
- Session cookie is the "non-standard data structure": `<username>%3a<32-char base62 token>`; anonymous = empty username (`session=%3a<token>`); after login `session=wiener%3aiqnGyq25rnENBH8etMHd63T7ZJy4XPQI`.
- **The app splits the cookie on the FIRST colon.** A payload containing a colon (e.g. `http://IP:PORT`) is silently cut, the token becomes garbage and the request degrades to a fresh anonymous session (302 + new cookie). This produced three false "payload was filtered" conclusions before the split rule was identified.
- Auth semantics: unknown username + own valid token -> **500 Internal Server Error** (no stack) on both `/my-account` and `/admin`; right username + wrong token -> 302 + new anonymous cookie; `/my-account?id=X` with X != cookie username -> 302 (session dropped). So the pre-colon part is looked up in a user table and the post-colon part in a session store; both must line up. `administrator:<our token>` -> 500 (not admin access).
- Comment surface: `POST /post/comment` (csrf, postId, comment, name, email, website) is immediately visible, no moderation. The `name` lands inside `<a id="author" href="<website>">` and is **HTML-escaped** (`<b>BX</b>` -> `&lt;b&gt;BX&lt;/b&gt;`), server-side limit "Name must be length 64 or less." Form-urlencoded note: a literal `+` in the payload body is decoded as a space.
- Injected payloads in the cookie's username part (raw and percent-encoded; `<script>` and `<svg/onload>` shapes, DNS-exfil and IP:9999-exfil) produced no OOB callback at all and no evidence of a privileged renderer for that value.
- OOB base (self-hosted, verified live): `ssh ubuntu@47.131.34.33`; authoritative zone `oob.dthack.io` (markers resolve via getent, e.g. `<marker>.oob.dthack.io` -> 47.131.34.33); DNS canary `dns_oob` writes `~/prs-oob/dns.log`; HTTP leg is `python3 http.server` on 9999 appending `/tmp/oob-http.log`; ports 80/9099 = `deaddrop`. Egress from this sandbox to :9999 times out (direct http_session to the listener fails) while ssh works, so the DNS leg is the dependable exfil channel from here.
- `oob_poll poll` blew its 60s deadline once with no output (ssh to that host occasionally stalls for ~50s; a short single-command retry answers in ~2s).

### Launch / tooling lessons
- `range_launch` needs `--widget-source <lab page path>`: the default `/web-security/` works for lab 1 but makes lab 2's launch 302 straight to `/web-security/` with no instance URL; with the lab's own path it returns the instance immediately.
- Jar discipline held: `cp /tmp/cj1.json /tmp/e1-lab1.json|/tmp/e1-lab2.json` then `range_launch --jar`; the portswigger.net `.AspNetCore.CookiesC1/C2` app session is the working key (auth0 is expired).
- Discipline breach to avoid next batch: while hunting lab 2's instance URL, the lab page was fetched with `http_session` (which does not strip solution blocks) instead of `page_read`, so the full official solution text was read into context. Use `page_read` for lab pages; raw fetches only for launch/widget surfaces.

### Open gaps
- Lab 1: no file-read sink found on any reachable endpoint; the stock endpoint's per-string deterministic integer is unexplained (mock/simulator behaviour) and nothing in the sqli/xss/cmdi/path battery moved it.
- Lab 2: unknown where the cookie username is stored/rendered for a privileged viewer; no bot callback observed.

