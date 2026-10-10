---
metadata:
  node_type: memory
name: "JWT Unverified Signature Lab"
description: "Lab solved: RS256 JWT with sub=administrator + original signature accepted (no verification); delete carlos via /admin/delete"
last_updated: 2026-10-10T12:03:19+08:00
created: 2026-10-10T12:03:19+08:00
---

## JWT authentication bypass via unverified signature (arm A, solved)

Target: /web-security/jwt/lab-jwt-authentication-bypass-via-unverified-signature
Instance: fresh launch (reused:false); logged in wiener:peter via http_session GET /login -> csrf -> POST /login; session cookie = RS256 JWT {kid, alg} / {iss, exp, sub}.

Method: server never verifies the signature. Kept the ORIGINAL signature segment verbatim, swapped only the payload to {"iss":"portswigger","exp":1791608560,"sub":"administrator"}. Payload JSON was exactly 60 bytes so base64 == base64url with no padding/+/ (b64 encode output usable as-is). Wrote the token by hand into a flat jar {"<host>":{"session":"<forged>"}} (format: host -> cookie name -> value) and sent GET /admin -> 200 admin panel; GET /admin/delete?username=carlos --follow -> banner is-solved, "User deleted successfully!".

Notes: no alg:none needed; signature strip not needed either. Crafting the payload segment via the b64 piece avoids needing a new JWT piece (candidate: fold into a jwt_mutate piece if reused).

