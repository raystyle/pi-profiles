---
metadata:
  node_type: memory
name: "arm-a lab-absolute-path-bypass solved"
description: "arm A: Path traversal absolute-path bypass solved via filename=/etc/passwd (no record needed beyond this)"
last_updated: 2026-10-10T19:34:24+08:00
created: 2026-10-10T19:34:24+08:00
---

## 2026-02-14 arm A: lab-absolute-path-bypass

- Range: /web-security/file-path-traversal/lab-absolute-path-bypass, widget-lab-id 9901AEA6...
- Instance fresh (reused:false): https://0a8400260335f9e980488f8700cc002d.web-security-academy.net/
- Method: `http_session get <base>/image?filename=/etc/passwd` -> 200, /etc/passwd body (2316 bytes, ~40 accounts).
  Traversal `../` is blocked but the parameter is treated as relative to a default working directory, so an
  absolute path passes through unmodified.
- Verdict: banner_verdict solved:true, congrats line present.
- Lesson: absolute-path bypass needs no encoding face - one request; `filename` param is the standard
  product-image read face on academy file-path-traversal labs.

