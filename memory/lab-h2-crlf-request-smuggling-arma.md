---
metadata:
  node_type: memory
name: "lab-h2-crlf-request-smuggling armA"
description: "A 臂实录: h2 CRLF 头值注入 + 超长 CL 饥饿 comment POST 吞下受害者 Chrome 请求, 读出 carlos 会话并接管, banner is-solved"
last_updated: 2026-10-09T21:09:54+08:00
created: 2026-10-09T21:09:54+08:00
---

题目: lab-request-smuggling-h2-request-smuggling-via-crlf-injection (advanced/h2 CRLF injection), 目标"gain access to another user's account", 受害者每 15s 访问首页。
实例: 0a99002004b7457881cc2b0500160033.web-security-academy.net (range_launch launch-url 直接起, reused:false)。

机制(实测四步):
1. 前端 h2->h1 降级不过滤头值: h2 `x-arm` 头值写 `x\r\n\r\n<完整 h1 请求>` 即整体注入, 后端照单执行(首证: 注入 POST /post/comment 精确 CL 存下 ARM1PROBE 评论)。
2. 前端在客户连接的生命周期内把上游连接绑成一条直连(同一 h2 连接的两个流会落到同一上游连接; --repeat 2 时 stream1 收到被走私 POST 的 302、stream3 收到本流 200)。
3. 上游连接在客户连接关闭后回到共享池, 受害者的请求会落到这条池化连接上(跨客户连接复用成立; 客户连接保持打开期间反而不会被别人用)。
4. 把走私 POST /post/comment 的 CL 写成 provided + 23(前端自补 `\r\nContent-Length: 0\r\n\r\n`)+ 悬垂 N: 后端停在读体, 受害者下一次首页请求的前 N 字节被当评论正文存下, 评论页即读出内容。悬垂 N 必须 <= 受害者请求长度(约 835B), N=800 截到 session 前 15 字符, N=830 取全。

武器与判读:
- h2_req 头值注入(NAME||VALUE 支持 \r\n 转义); 手动试错用 h2_req, 定稿件为 .pi-rs/rust-scripts/h2_crlf_capture.rs。
- 陷阱1: 提供的 body 里 `&comment=` 后紧跟裸 CRLF 会让写入端整条拒绝(评论不落库), 标记必须放在注释值内(如 `%23r1.1%23`)。
- 陷阱2: 悬垂过大(>=900)不会完成, 悬垂过小只截到请求头部; 用 830 一次命中。
- 读回: 评论正文含 `cookie: victim-fingerprint=...; secret=...; session=swakQt9iPiBdgCzaHvT25LYdk697xMGk`。

结果证据:
- 窃得 cookie `session=swakQt9iPiBdgCzaHvT25LYdk697xMGk`(+ victim-fingerprint/secret), GET /my-account 返回 "Your username is: carlos", banner `is-solved`, congrats "Congratulations, you solved the lab!"。
- 定稿件复跑: captured.session=swakQt9iPiBdgCzaHvT25LYdk697xMGk, verdict.verified=true, account_snippet "Your username is: carlos"。

未解疑点(留档): 前端自补 `\r\nContent-Length: 0\r\n\r\n` 后紧跟的 33B `GET / HTTP/1.1\r\nHost: <host>` 来源未证; 悬垂 34-60B 可秒完成, 说明该字节来自连接上的下一次写。

