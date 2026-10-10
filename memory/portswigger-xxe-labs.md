---
metadata:
  node_type: memory
name: "PortSwigger XXE Labs"
description: "XInclude XXE lab solve: payload in productId form field of /product/stock; 400 body leaks /etc/passwd; banner solved"
last_updated: 2026-10-10T01:14:33+08:00
created: 2026-10-10T01:14:33+08:00
---

## 2026-xx lab-xinclude-attack (XInclude to retrieve files)

- Instance: launched via range_launch launch-url https://portswigger.net/web-security/xxe/lab-xinclude-attack --jar /tmp/cj1.json (page_read gave lab_id 68BB9B0D... directly; no /api/widgets fallback needed).
- Sink: POST /product/stock, form-urlencoded body productId & storeId (not JSON, not raw XML) - the server embeds productId into its own server-side XML document, so a classic DOCTYPE/DTD is impossible.
- Payload placed in the productId form value, URL-encoded by http_session --form:
  `<foo xmlns:xi="http://www.w3.org/2001/XInclude"><xi:include parse="text" href="file:///etc/passwd"/></foo>`
- Result: HTTP 400 with JSON body "Invalid product ID: <file contents>" - the file dump is returned even on error status; judge the body, not the status.
- Verdict: banner_verdict on instance root -> solved:true, "Congratulations, you solved the lab!".
- Lesson: for XInclude labs the injection point is the form field whose value is interpolated into server XML; parse="text" is required to keep the file as text.

