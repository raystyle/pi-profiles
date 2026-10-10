---
metadata:
  node_type: memory
name: "WCD lab exact-match cache rules arm A"
description: "WCD exact-match cache lab: Tomcat matrix-param + cache %2f/dot-segment normalization caches the admin page under /robots.txt, steal CSRF, deliver CSRF form"
last_updated: 2026-10-10T15:49:56+08:00
created: 2026-10-10T15:49:56+08:00
---

## 2026 - lab-wcd-exploiting-exact-match-cache-rules (solved)

Target: PortSwigger Web Security Academy, WCD exact-match cache rules. Goal: change administrator's email; creds wiener:peter. Instance 0a61008204ad75a880b60d9f000c0094.

Cache behaviour mapped by cache_probe x_cache oracle (query is part of the cache key, TTL 30 s):
- `/robots.txt` and `/favicon.ico` are the only cacheable paths (X-Cache miss then hit, Cache-Control max-age=30). `/resources/*`, `/my-account`, and static assets are NOT cached.
- The cache DECODES `%2f` and resolves dot-segments; the origin (Apache-Coyote, Tomcat-style) does NOT. `/my-account/..%2frobots.txt` -> cache normalizes to `/robots.txt` (cached), origin 404.
- Tomcat strips `;matrix-params`; the cache keeps them. Exploit URL: `/my-account;%2f..%2frobots.txt` -> cache normalizes to `/robots.txt` (rule matches, stores the response), origin serves `/my-account`.

Exploit chain:
1. Deliver 302 -> `/my-account;%2f..%2frobots.txt` via exploit server; the admin's /my-account page (with csrf) is cached under normalized key `/robots.txt`.
2. Retrieve with plain GET /robots.txt (or the crafted URL) -> X-Cache hit, body shows "Your username is: administrator" + csrf token. TTL 30 s, fetch fast.
3. Token is session-bound (using it with wiener's session -> 400 "Invalid CSRF token"). The admin bot's session persists across deliveries, so the leaked token stays valid.
4. Deliver an auto-submit POST form to /my-account/change-email (email + leaked csrf) -> admin's browser submits with its own session -> solved.

Pieces: page_read, range_launch, http_session, http_dump, cache_probe (x_cache/path-normalization oracle), banner_verdict. Delivery: exploit server POST / with urlIsHttps + responseFile + responseHead + responseBody + formAction=DELIVER_TO_VICTIM, --follow.

