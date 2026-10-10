---
metadata:
  node_type: memory
name: "Client-side desync lab solve"
description: "Client-side desync lab solved: POST / ignores Content-Length; comment gadget captures victim Cookie; no-cors fetch chain + exploit-server delivery"
last_updated: 2026-10-10T22:25:47+08:00
created: 2026-10-10T22:25:47+08:00
---

## Lab solve: client-side desync (PortSwigger, /web-security/request-smuggling/browser/client-side-desync)

- **Vector**: app is language-prefixed (`/en/...`); `POST /` ignores a declared `Content-Length` and answers `302 /en` immediately, so body bytes left in the socket are parsed as the next request. Proven with `conn_reuse`: one connection, `POST /` + `Content-Length: 500` whose body is a `GET /404probe` → two responses (302, then 404).
- **Gadget**: comment form `POST /en/post/comment` (fields `csrf`, `postId`, `comment`, `name`, `email`, `website`). CSRF is session-bound (`"Invalid CSRF token (session does not contain a CSRF token)"`) and `name`/`email` are required server-side (`"Missing parameter"`) — so the smuggled request must embed a valid `Cookie: session=<ours>` plus our matching csrf, and the body must carry `name`/`email`/`website` BEFORE `comment=`.
- **Exploit**: exploit-server page runs `fetch(L+'/', {method:'POST', body:<partial POST /en/post/comment with Content-Length: 1000>, headers:{'Content-Type':'text/plain;charset=UTF-8'}, credentials:'include', mode:'no-cors'})` then several follow-up `fetch(L+'/en/', {credentials:'include', mode:'no-cors'})`. Request bytes after `comment=` land in the stored comment; `Content-Length: 1000` reliably captured the whole `Cookie:` line (incl. `session=`, `secret=`, `victim-fingerprint=`).
- **Delivery**: exploit server needs `responseFile=/exploit` + `urlIsHttps=true` + `responseHead` + `responseBody`; `formAction=STORE` then `DELIVER_TO_VICTIM` with `--follow` (302 → /deliver-to-victim).
- **Verdict**: stolen victim session in a jar → `banner_verdict <base> --jar` returns `solved:true`; the flag flips on the first request that carries the victim session, and the banner in that same response may still read "Not solved" — re-read before doubting.
- Exploit-server fetch must use `mode:'no-cors'` cross-origin: a CORS-mode fetch whose response lacks ACAO rejects and breaks the follow-up chain that fills the hungry body.

