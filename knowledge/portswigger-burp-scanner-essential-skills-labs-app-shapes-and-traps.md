---
title: "PortSwigger Burp Scanner essential-skills labs - app shapes and traps"
---

# PortSwigger Burp Scanner essential-skills labs - app shapes and traps

# PortSwigger Burp Scanner essential-skills labs - app shapes and traps

Two Academy labs under `/web-security/essential-skills/using-burp-scanner-during-manual-testing/` exist purely to teach "Scan a specific request / Scan selected insertion point". Both are reachable and attackable without Burp; the app behind each is a stock clone with one hidden weakness. Facts below are the current state after direct probing.

## Launch

Instance delivery is pure HTTP via [[lab-launch]]. `range_launch` must be given `--widget-source <the lab's own page path>`: with the default `/web-security/` the *targeted-scanning* lab launches fine, but the *non-standard-data-structures* lab 302s straight back to `/web-security/` and yields no instance URL. Key is the `portswigger.net` `.AspNetCore.CookiesC1/C2` app session in a per-instance jar copy.

## Lab: "Discovering vulnerabilities quickly with targeted scanning" (read /etc/passwd, 10-minute timer)

Shop clone. Reaches: `/`, `/product?productId=N`, `POST /product/stock`, `/image/<path>`, and an unlinked `/filter` that returns the listing page.

- The only dynamic endpoint is `POST /product/stock`. It takes exactly the two form fields `productId` and `storeId`, ignores `Content-Type` (a urlencoded body is still parsed when labelled `application/json`), and rejects any XML or JSON body with 400 `"No such product or store"`. `GET /product/stock` -> 405.
- With *any* values it answers a plain integer 0-999 that is a deterministic function of the input string: `1`->732, `/etc/passwd`->898, `/dev/null`->217, `AAAAA`->941, and each string repeats the same number on a freshly relaunched instance. Equivalent file paths (`/etc/passwd`, `/etc//passwd`, `/etc/./passwd`) return different numbers, so the value is not filesystem-derived.
- No file-read sink is reachable: `/image/` rejects literal and encoded traversal (`..`, `..%2f`, `..%252f`, `%2e%2e%2f`, double-encoding) with a JSON 404, `/image?filename=` does not exist, `/resources/` traversal 404s, and `/product?productId=` answers 400 `"Invalid product ID"`.
- Nothing in the sqli / xss / cmdi / path space moves the endpoint: quoted SQL payloads, `SLEEP`/`pg_sleep`/`LOAD_FILE`/boolean pairs, `;sleep 8` and `| sleep 8` (no delay), template syntax (`{{7*7}}`, `${7*7}`, `#{7*7}`, `<%= %>`), and session-cookie / `Accept-Language` traversal all behave as the opaque-integer case. A 40-name endpoint wordlist surfaces nothing beyond the routes above.
- The lab timer is enforced by the app: at expiry the page becomes "Time's up!" with a `GET /try-again` link (302 -> `/`) that restarts the instance; re-launching through `range_launch` returns the existing instance.

## Lab: "Scanning non-standard data structures" (wiener:peter, delete carlos)

Blog clone: `/`, `/post?postId=N`, `POST /post/comment`, `/login` (hidden `csrf` field), `/my-account`, `/my-account/change-email`, `/admin`, `/logout`.

- The "non-standard data structure" is the session cookie: `<username>%3a<32-char base62 token>`. Anonymous is an empty username with a token; after login the value is e.g. `wiener%3aiqnGyq25rnENBH8etMHd63T7ZJy4XPQI`.
- **The app splits that value on the FIRST colon.** Any payload containing a colon (`http://IP:PORT`, base64, etc.) is cut in half: the username becomes the fragment before the colon and the token becomes the rest, so the request degrades to a fresh anonymous session (302 plus a new cookie). Detect this before concluding a payload was filtered.
- Auth semantics, all four observed: cookie username known + token valid -> the account page renders; cookie username unknown + valid token -> 500 `Internal Server Error` (no stack) on both `/my-account` and `/admin`; known username + invalid token -> 302 + new anonymous cookie; `/my-account?id=X` when `X` differs from the cookie username -> 302 (session dropped). The pre-colon part is looked up as a user, the post-colon part as a session; both must line up, and `administrator:<someone else's token>` is a 500 rather than admin access.
- Comments post straight to `POST /post/comment` (`csrf`, `postId`, `comment`, `name`, `email`, `website`) and are public immediately, with no moderation step. `name` renders inside `<a id="author" href="<website>">` and is HTML-escaped (`<b>BX</b>` -> `&lt;b&gt;BX&lt;/b&gt;`); the server enforces "Name must be length 64 or less." In form-urlencoded bodies a literal `+` decodes to a space, which silently mangles payloads.
- `/admin` answers 401 for a plain wiener session.

## Tool coverage notes

- `dbg_cli net scan` battery families are exactly `sqli,xss,cmdi,path` (anything else -> "empty battery"); it captures the live request as a template with slots for the cookie, each form field and selected headers, and skips payloads that are unsafe for a cookie slot. Zero `suspects` therefore only rules out those four families on the slots it tested - XXE, SSRF, SSTI and deserialization need a different instrument.
- `net scan` needs the templated request to fire inside its wait window: start it in the background, then drive the page. It refuses a pattern that `net intercept` already has armed.
- OOB for exfil: the self-hosted base is an authoritative DNS canary for `oob.dthack.io` (markers resolve) plus an HTTP listener on `:9999`; from the sandbox the DNS leg is the dependable one, since direct egress to `:9999` times out.

## Links

- depends-on: [[lab-launch]]
