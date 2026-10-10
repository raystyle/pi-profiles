---
metadata:
  node_type: memory
name: "lab-multistep clickjacking arm A"
description: "Multistep clickjacking solved on first delivery: server-rendered two-step delete + geometry measured inside a 780x1000 same-origin iframe"
last_updated: 2026-10-10T20:15:37+08:00
created: 2026-10-10T20:15:37+08:00
---

## 2026-10-10 lab-multistep (clickjacking) arm A - solved

Instance: https://0a7f00990446118b80fb8f6d00ab00c4.web-security-academy.net (fresh, reused:false);
exploit server: exploit-0a6e00c40423117d80a68e8f01440011.exploit-server.net.

Trajectory / facts:
- page_read of /web-security/clickjacking/lab-multistep gave the method directly (delete button + confirmation dialog, two decoy actions, element count two).
- The "confirmation dialog" is NOT client-side: /my-account has no script beyond labHeader.js and no dialog markup. POST /my-account/delete (any csrf) returns a server-rendered confirm page ("Are you sure?", hidden csrf + confirmed=true, a.button "No, take me back", button "Yes"); the second POST with confirmed=true performs the deletion. So the victim's single iframe walks my-account -> delete POST -> confirm page, and the two decoys must sit over two different positions in the same iframe. GET /my-account/delete = 405 (safe probe), POST with a bogus csrf is non-destructive and reveals the confirm page.
- Geometry: measured in a same-origin iframe appended to the loaded lab page (iframe.contentDocument readable, no srcdoc/fetch needed) at exactly the exploit iframe size 780x1000 -> delete button rect (x16,y481.1,168x33); after clicking delete inside that iframe, Yes button rect (x209.7,y276.9,120x33). Absolute decoys at those rects + iframe opacity 0.0001 z-index 2 (iframe on top receives the clicks, decoys visible underneath) solved it on the first DELIVER_TO_VICTIM.
- Exploit server POST / fields: urlIsHttps=on, responseFile=/exploit, responseHead, responseBody, formAction=STORE|DELIVER_TO_VICTIM; STORE echoes the stored body (self-verifying).

Blockers hit:
- browser_suite Runtime.evaluate wedged ("cdp timeout: deadline has elapsed") on every later call after Emulation.setDeviceMetricsOverride + an awaitPromise eval that never resolved; recovery = browser_suite stop (pkill by profile) then re-goto. Prefer load-event state + synchronous read-back evals over awaitPromise.
- Emulation device-metrics override is not needed: just size the measurement iframe to the exploit iframe size.
- Killing chrome (browser_suite stop) drops session cookies set via cookies import unless an explicit expires was passed; re-import with "expires":<epoch> for measurement sessions.

Reusable piece gap: no exploit-server piece exists; STORE/DELIVER/ACCESS_LOG all went through http_session --form (fine, but a dedicated piece would cut the 4-call dance).

