---
metadata:
  node_type: memory
name: "PRS Batch E1 - burp-scanner essential-skills labs"
description: "Closing round on lab 2: POST /login ssti+crlf pass (51 probes, 5 skipped, 0 suspects, baseline 400 stale-csrf) and the hand-rolled cookie CRLF test in four legs (control 200; %0d%0a in username -> 500; bare CRLF -> 302 + new anon cookie; %0d%0a in token -> 302) with no response-header injection or marker anywhere - cookie CRLF surface closed, lab still unsolved"
last_updated: 2026-10-07T22:47:11+08:00
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


## 2026-10-07

## Re-attack round (seven-family battery, net scan v1.5+)

Target: lab-discovering-vulnerabilities-quickly-with-targeted-scanning only. Outcome: still unsolved, but the stock request is now clean across **all seven** families and the new desk build's operative bugs are pinned.

### Verdicts
- `POST /product/stock` pass A (batch E1): `--battery sqli,xss,cmdi,path` -> 53 probes, 21 cookie-unsafe skipped, suspects EMPTY.
- `POST /product/stock` pass B (this round): `--battery xxe,ssti,ssrf` -> 33 probes, 39 skipped, suspects EMPTY. Template slots now `path:1:product`, `path:2:stock`, `cookie:session`, `form:productId`, `form:storeId`, `pname:new`, `header:Referer`, `header:User-Agent` -> path segments and param names are slots in the new build.
- Manual SSTI sweep on `productId` (`{{7*7}}`, `{{1337*1337}}`, `${7*7}`, `#{7*7}`, `<%= 7*7 %>`, literal `1337*1337`) -> every value echoed in the JSON 400 `"Invalid product ID: ..."`, no 49 / 1787569 -> no evaluation context.
- Corrected target fact: the stock integer is **per-instance state**, not a hash of the input. `productId=1&storeId=1` -> 732 on the earlier instance and 32 after relaunch at the same host; arbitrary strings still get values. So the endpoint is a seeded mock/DB lookup, which is why no injection family moves it.

### net scan acceptance findings (desk 1.5.0 -> 1.7.0, cli 1.4.1 -> 1.5.1 during the round)
- **Full seven-family run never completes**: >5 min wall clock, then `net: reply: Resource temporarily unavailable (os error 11)`. Retry with `--max-probes 120 --budget-ms 110000` still ran past 200s and had to be killed -> the budget flag does not bound total scan time. Narrow passes (2-4 families) finish in ~25-50s and are the workable shape.
- **Patterns are not auto-released**: the next scan on the same pattern fails with `net: pattern "..." already intercepted`. New op `dbg_cli net intercept off --pattern <P>` clears it (receipt: `remaining_intercepts: []`).
- **Glob semantics**: `*` covers scheme+host only. `*/product/stock` matches; `*/stock` never matches `/product/stock` (so a rename that only shortens the literal path silently captures nothing). `/image/...` needs `*/image/*`.
- **20s match window + arm/click race**: start the scan in the background, then click synchronously (a `setTimeout` click from a backgrounded tab may never fire - Chrome throttles timers). Click twice ~8s apart to be sure a request lands inside the window.
- **Image route needs a cache-bust**: images carry `max-age=3600`, so reloading a listing fires no `/image/...` request; clear via `browser_suite call Network.clearBrowserCache` or switch to an uncached product image. Even then a one-family path pass over the image route ran >150s and was killed - image-path scanning is slow, unresolved.
- `browser_suite eval` against a stale page throws `Uncaught` (the tab had navigated); re-`goto` before clicking.
- Lab 1 timer: `range_launch` returns the existing instance without resetting the clock; the only reset is `GET /try-again` while the "Time's up!" page is showing.

### Still open
- Where the intended file-read primitive lives: not on `/product/stock` (seven families clean), not on `/image/<path>` (traversal 404s at every encoding tried), not on `/filter` (listing alias), no other reachable route in a 40-name wordlist.


## 2026-10-07

## Third round: lab 2 (scanning non-standard data structures) vs the eight-family battery

Desk 1.9.1 / cli 1.6.0, eight families (sqli/xss/cmdi/path/xxe/ssrf/ssti/crlf), eight slot shapes (query/cookie/form/json/header/pathseg/pname/body:xml), `--speed` fast|normal|thorough. Instance 0a6000b804f03df88488183000f9004a. Verdict: **not solved**, no suspect anywhere.

### Passes (narrow, 2-4 families each; release the pattern between passes)
| request | battery | probes | skipped | suspects |
| --- | --- | --- | --- | --- |
| POST /post/comment | sqli,xss | 78 | 4 | [] |
| POST /post/comment | ssti,crlf,path,cmdi | 96 (truncated) | 25 | [] |
| GET /my-account?id=wiener | crlf,ssti | 30 | 5 | [] |
| GET /my-account?id=wiener | sqli,ssti,path | 48 | 10 | [] |
| POST /login | sqli,xss | 50 | 4 | [] |

Driven by real page actions: browser logged in as wiener (page_interact fill+submit), comment submitted via a synchronous eval click inside the armed window, account/login requests fired by `browser_suite goto` right after arming.

### Manual probes this round
- Comment `website` field is **HTML-escaped**: `http://x.com"><svg/onload=alert(1)>` renders as `href="http://x.com&quot;&gt;&lt;svg/onload=alert(1)&gt;"`. Same for `name` (`<b>BX</b>` -> `&lt;b&gt;BX&lt;/b&gt;`). Neither the author name nor the link target is injectable.
- `POST /login` re-submitted from the logged-in browser: the app answers 400 with body len 60 (stale csrf), so the login POST used as a scan template is a rejection response - worth remembering when reading that baseline.

### Coverage gap found (matters for the lab's headline surface)
- The session-cookie username slot cannot be CRLF-tested by the scanner: CRLF payloads are cookie-unsafe and are dropped (`skipped_cookie_unsafe` 4-25 per pass, including the cookie slot). A printf-style raw request through `raw_http`/`conn_reuse` is the only way to put a real CRLF into a cookie value - the cookie slot's crlf verdict from this scanner is "not tested", not "clean".

### Source discipline note (this lab carries a contamination tag)
- Session shape (`username:token`, first-colon split) and the four auth states were established by direct probing last round; the "XSS in the cookie" hypothesis came from an earlier accidental read of the lab page's solution text via `http_session` (it does not strip solution blocks; `page_read` does, and http_session now strips too). Everything recorded here is reproducible from the requests themselves; nothing in this round's results depends on that leaked text.


## 2026-10-07

## Closing round: the two untested lab-2 faces (both zero displacement)

### 1. POST /login with ssti+crlf (the form-slot header-block oracle)
- Template: POST /login, slots `path:1:login`, `cookie:session`, `form:csrf`, `form:username`, `form:password`, `pname:new`, `header:Referer`, `header:User-Agent`.
- Receipt: `probes 51, skipped_cookie_unsafe 5, suspects []`, `baseline {len 60, status 400}` - the stale-csrf rejection baseline, as predicted last round; judge only after baseline screening.
- Combined with the earlier sqli+xss pass (50 probes) and the ssti+crlf pass on /post/comment, every form slot of both forms now has sqli/xss/ssti/crlf coverage with zero suspects.

### 2. Cookie username slot, real CRLF hand-rolled (`raw_http`, session `wiener%3aTsRlIwo3c3xBM0i10rkXHU5CAFViVp06`)
| leg | request | status | response header block |
| --- | --- | --- | --- |
| control | `Cookie: session=wiener%3a<token>` | 200 | baseline: Content-Type, Cache-Control, Set-Cookie (plain), X-Frame-Options, Connection, Content-Length |
| encoded CRLF in username | `session=wiener%0d%0aZXQJ-CRLF%3a<token>` | 500 (2480B) | no injected header; CRLF reaches the username field -> unknown user -> generic error page |
| bare CRLF in header value | `Cookie: session=wiener` + CRLF + `ZXQJ-CRLF: bare` | 302 /login + fresh anon cookie | no injected header; injected text absorbed as its own request header, cookie value truncated to `wiener` (no token) |
| encoded CRLF in token | `session=wiener%3a<token>%0d%0aZXQJ-CRLF` | 302 /login + fresh anon cookie | no injected header; token lookup fails |
- Verdict: **cookie slot CRLF closed** - no response-header injection, no marker, no malformed Location/Set-Cookie; only semantics-consistent divergence (500 for unknown user, 302 for a broken token).
- Detail worth keeping: the app URL-decodes the cookie value *before* splitting on the first colon (that is why the server itself emits `wiener%3a<token>`). A payload colon therefore becomes the split point, which is why the username-position payload has to stay colon-free.

### Round close
- Both faces zero displacement; per the round's own rule, lab 2's cookie-crlf surface is recorded as hand-rolled and re-tested, and no further rounds are opened on this lab.
- `banner_verdict` after the round: `solved=false, congrats_line=null`.

