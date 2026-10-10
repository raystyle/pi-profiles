---
metadata:
  node_type: memory
name: "Web Cache Poisoning Multi-Header Lab"
description: "lab-web-cache-poisoning-with-multiple-headers (arm A) solved; cross-origin alert did not count, same-origin script URL poisoning did"
last_updated: 2026-10-10T16:27:06+08:00
created: 2026-10-10T16:27:06+08:00
---

### 2026-10-10 lab-web-cache-poisoning-with-multiple-headers (arm A) — solved

Instance: 0a40008804b3f0b881384e79000200da.web-security-academy.net (exploit server exploit-0a8d00a9043ef091813b4d9c0182006d.exploit-server.net).

Gadget confirmed: `X-Forwarded-Host: <anything>` + `X-Forwarded-Scheme: <not-https>` → 302 `Location: https://<X-Forwarded-Host><request-URI>`; either header alone → normal 200, no body reflection of the host anywhere. Cache stores the 302 (max-age=30, X-Cache miss→hit), home page and /resources/* both cacheable.

Failed route (important): poison `/` so the victim is redirected to the exploit server, payload `<script>alert(document.cookie)</script>`. The victim browser really did execute it — proven by an added `fetch('/exec-probe')` beacon seen in the exploit-server access log from the Victim UA, four consecutive visits — but the lab stayed unsolved. Cross-origin alert (exploit-server.net) does not count, and the alert argument would be empty there.

Winning route: poison the SAME-ORIGIN resource the cached home page loads — `/resources/js/tracking.js` — so the script request gets the 302 and the script body comes from the exploit server but executes in the lab origin (alert carries the real cookie). Two details make it work: store the payload at the path that mirrors the redirect Location (`responseFile=/resources/js/tracking.js`, head `Content-Type: application/javascript`, body `alert(document.cookie)`), because the exploit server serves only that exact path and 404s deeper ones; and loop the poison request on the exact key `/resources/js/tracking.js` (no query) faster than the 30s TTL. Solved within one victim cycle.

Exploit-server quirks: POST needs responseFile + responseHead + responseBody (missing any → 400 "Missing parameter"); `responseFile=/` → 400 "footgun detected" (root reserved for the admin UI); ACCESS_LOG also needs all three, harmless when repeating the stored values; log readable at GET /log.

