---
metadata:
  node_type: memory
name: "prs arm-A lab-validate-file-extension-null-byte-bypass"
description: "arm-A solve of the null-byte extension-bypass path traversal lab: /image?filename=../../../etc/passwd%00.png, 4 calls, banner solved"
last_updated: 2026-10-10T19:36:50+08:00
created: 2026-10-10T19:36:50+08:00
---

## 2026-02-__ arm A: file path traversal, extension validation null-byte bypass

Path: /web-security/file-path-traversal/lab-validate-file-extension-null-byte-bypass
widget-lab-id: 563206D375AAE19B45B636FBD08FAF36892E8229B2F84D5908110BBDB7798561
Instance: https://0ab100cd041551ad8034712d00f500a0.web-security-academy.net/ (reused:false, fresh, no residual state)

Shape: /image?filename= serves product images; the app validates that the filename ends in the
expected extension, so a plain ../ cannot reach /etc/passwd. Old PHP/C-style null-byte truncation
remains: the validator sees the ".png" suffix, the filesystem open call stops at the NUL.

Winning request (ONE call, http_dump):
  GET /image?filename=../../../etc/passwd%00.png
  -> 200, body_len 2316, content-type image/png, body starts "root:x:0:0:root:/root:/bin/bash"

Depth note: /var/www/images/ + ../../../et...c/passwd = 3 ups; %00 must stay percent-encoded on the
wire (do not let the client decode it into a raw NUL before the request line).

Verification: banner_verdict on instance root -> solved_class true,
congrats_line "<h4>Congratulations, you solved the lab!</h4>".

Cost: 4 tool calls (page_read lab page, range_launch, http_dump, banner_verdict). No piece needed
beyond http_dump - the built-in client preserved %00 in the path unchanged.

