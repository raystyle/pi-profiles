---
title: lab-request-smuggling-0cl-request-smuggling
---

# lab-request-smuggling-0cl-request-smuggling

> evidences: [[h2-smuggling-family]]

- 题面:0.CL request smuggling;Carlos 每 5s 开首页,让他执行 `alert()`。
- 实例(批47):https://0aec00c803f8f74680a076ff00ab0056.web-security-academy.net
- 判定:**stuck**(第三轮仍未能造出 0.CL 帧;缺整套双 desync 编排)

## 新证据(新实例复核)

1. **畸形头 `Content-Length : N`(冒号前空格)在全新实例上依旧不产生帧长分叉**:一次 write 发 `POST /resources/css/anything`(带 `Content-Length : 92`)+ 紧跟 `GET /404probe` ⇒ 同连接**两条完整响应**(302 `Location: /resources/css/anything/` 130B + 404 `"Not Found"` 244B)⇒ 前后端帧长读法一致(无人 holding),该畸变头不是本实例的 0.CL 原语。与批44 结论一致,**跨实例稳定**。
2. 复核出的新细节:两条响应的 `Keep-Alive` 都是 **`timeout=10`**(不是 batch 43/44 在 tunnelling 题里见到的 `timeout=0`),且第二条响应带**新的 `Set-Cookie: session=…`** ⇒ 前端按"持久连接上的两条独立请求"处理,确认没有请求被吞。
3. early-response gadget 仍在:静态目录路径 `/resources/css/anything` 立即回 302(不等 body)。
4. 首页仍**零 JS、无 exploit-link**;唯一 XSS gadget 仍是 `/post?postId=N` 把 **User-Agent 原样**写进 `<input type="hidden" name="userAgent" value="…">` ⇒ 交付面必须是"把这条响应投给受害者的 `/` 请求"(而非自产 HTML)。
5. 题面确认设计源:描述直接引用 PortSwigger Research《HTTP/1.1 Must Die》("This lab is based on real-world vulnerabilities discovered by PortSwigger Research"),与在档外部形状(双 desync:stage1 / stage2_chopped+revealed+smuggled)同源。

## 未决面

- 缺的仍是**双 desync 的字节算术**:`stage1` 制造 holding、`stage2_chopped/revealed` 精确切分、再把 `GET /post?postId=8` + `User-Agent: a"/><script>alert(1)</script>` 投到 Carlos 的 `/` 上;本实例并不响应 `Content-Length :`(空格)畸变,所以外部 writeup 的第一跳需要换一个 holding 原语(候选:h2 降级侧 `content-length` 名字畸变、`Transfer-Encoding` 与 CL 并存、obs-fold)。
- 15 分钟单题上限内无法把整套编排跑通;下一手应先造"holding 探测件"(逐畸变头测同连接响应条数),再谈编排。

## 复现命令

```
range_launch launch 4BAD74C3…EFC8CC66 --jar ~/.pi-rs/agent/chrome-jar.json
conn_reuse "https://<inst>/" --send-str 'POST /resources/css/anything HTTP/1.1\r\nHost: <inst>\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length : 92\r\n\r\nGET /404probe HTTP/1.1\r\nHost: <inst>\r\n\r\n'
```

## 关系

- 族:[[h2-smuggling-family]]、[[request-smuggling-family]];方法见 [[h2-tunnelling-and-h2cl-practice]]。
