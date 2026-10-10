---
metadata:
  node_type: memory
name: "prs-arm-a-lab-lab-confirming-te-cl-via-differential-responses"
description: "A 臂实录:TE.CL 差分响应确认题——不完整走私请求 + 新连接 GET / 读回 404,raw_matrix 交错变体一次成"
last_updated: 2026-10-09T14:53:35+08:00
created: 2026-10-09T14:53:35+08:00
---

# arm A: lab-confirming-te-cl-via-differential-responses

实例: 0a5200d50400fa0e80dd497400a500ac.web-security-academy.net (range_launch launch-url 直接吃 canonical 路径, 一次成功)
目标: 走私一条请求到后端, 使随后对 / 的请求收到 404。

## 前端/后端面(实测)
- 前端认 Transfer-Encoding: chunked: 给 CL+TE 但体不是合法 chunk 格式 → 400 (desync_probe 的 cl-plus-te 行即此)。
- 前端对每个响应都回 Connection: close; 同一条 TCP 上第二笔请求永不被处理 (conn_reuse --sequential 空白对照证实)。
  => 观测通道在池子: 毒化的上游连接被前端回收, 由"另一条新连接"的请求去命中。

## 已成形的帧 (TLS 上 HTTP/1.1)
POST / HTTP/1.1 + Host + Content-Length: 4 + Transfer-Encoding: chunked, 空行, 体 =
  `<hex(len(smuggled))>\r\n` + smuggled + `\r\n0\r\n\r\n`
smuggled = "GET /404 HTTP/1.1\r\nHost: <lab>\r\nContent-Length: 20\r\n\r\n"

## 关键判据
- smuggled 必须"不完整" (CL 大于其体) —— 后端挂起等体, 上游连接对前端仍是干净可回收的。
- 若把走私请求写完整 (CL 恰好吞掉 \r\n0\r\n\r\n 尾巴), 前后端各回一个响应: 原请求 200、后续请求仍 200, 差分不可见。
- 命中: 新连接的 GET / 被写进毒化流, 补完 smuggled 的体, 读到 smuggled 的 404 (11 字节 "Not Found")。
- follow1 = 404 之后 banner_verdict 即 solved:true, congrats "<h4>Congratulations, you solved the lab!</h4>"。

## 器材
- raw_matrix 一个 spec 里交错 [毒帧, GET /, GET /] 即可, 每变体各自新连接, 一个信封看全差分。
- 坑: raw_matrix 的 {{HOST}} 只替换 line/headers, 不替换 body; body 里必须写字面主机名, 否则 hex 与 chunk 数据长度不符 → 前端 400。
- conn_reuse --te-cl 的构造 (CL=len(hex)+2) 只能做"完整走私请求"那一形, 本题要的是不完整形, 故走 raw_matrix 手拼字节。

