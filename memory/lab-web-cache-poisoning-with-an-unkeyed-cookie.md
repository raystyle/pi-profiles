---
metadata:
  node_type: memory
name: "Lab: Web cache poisoning with an unkeyed cookie"
description: "Unkeyed `fehost` cookie reflected into a JS block; poisoned `/` with `xyz\"-alert(1)}//` and sustained re-poisoning, solved"
last_updated: 2026-10-10T16:04:23+08:00
created: 2026-10-10T16:04:23+08:00
---

## 2026-02-14 lab-web-cache-poisoning-with-an-unkeyed-cookie (arm A) - solved

Instance: https://0a57007f048d722d81ae760600f00065.web-security-academy.net/ (fresh, reused:false)

Recon:
- `page_read` on the canonical academy path returns `lab_id` D131A4D5F8A6A7E2AE584E9BC217208C223CAD4BAC3AED016482305228D68FB2; `range_launch launch-url <path> --jar /tmp/cj1.json` launched directly.
- Home page body carries `data = {"host":"...","path":"/","frontend":"prod-cache-01"}` inside a `<script>` block; response sets `Set-Cookie: fehost=prod-cache-01; Secure; HttpOnly`.
- Sending `Cookie: fehost=<marker>` changes only the `frontend` value -> the cookie is reflected unescaped into a JS string, and the cookie is NOT part of the cache key (`Cache-Control: max-age=30`, `X-Cache: miss|hit`).

Exploit:
- Payload `fehost=xyz"-alert(1)}//` renders `"frontend":"xyz"-alert(1)}//"}` -> object closes, `alert(1)` runs, `//` comments the tail.
- Key discipline: the victim visits `/` itself, so the poisoned key must be `/` - a cache-buster query (`/?x=1`) gets its own key and never reaches the victim. Wait past max-age (nap 34s) so the origin is reached, then sustain with `poison_loop <base>/ --header 'Cookie: ...' --interval-secs 8 --count 24` (background), which re-poisons `/` on every expiry.
- Verified: plain GET of `/` returned `X-Cache: hit`, `age: 19`, body containing `alert(1)`; `banner_verdict` after one victim cycle reported `solved:true` + `Congratulations, you solved the lab!`.

Lessons:
- Unkeyed-cookie reflection in a JSON `<script>` block: the JSON string break needs `"-` (close string, arithmetic operator) plus `}` to close the object and `//` to comment the remainder.
- When the victim has a fixed entry URL, never cache-bust that URL in the poisoning request; instead pace re-poisoning across the max-age window.

