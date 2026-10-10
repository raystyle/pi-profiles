---
metadata:
  node_type: memory
name: "Request Smuggling CL.TE Front-End Header Reveal"
description: "CL.TE 三段链:不完整体的 smuggled 请求借 follow 连接读回响应,反射前端加头后再带该头走私 /admin 与 delete"
last_updated: 2026-10-09T15:12:16+08:00
created: 2026-10-09T15:12:16+08:00
---

## 目标与结论
PortSwigger lab-reveal-front-end-request-rewriting 解到 congrats(A 臂)。实例 https://0a68005f038fc09980278f2200420029.web-security-academy.net/(range_launch launch-url 一次起号,jar /tmp/cj1.json)。
CL.TE:前端不支持 chunked(用 CL),后端 TE 优先。

## 关键机制(复用点)
- 读回通道 = smuggle_seq 的「不完整请求体」形状:smuggled 请求留一个小的 Content-Length(件把它改写成 body.len()+deficit),后端读完自己的头后仍在等 body;前端随后转发的 follow 请求字节正好补上,后端便把它当独立请求处理,响应队列错配(poisoning)把这条 smuggled 响应投给 follow 连接 —— 即件里的 `target`。deficit 20 命中。
- 反例:smuggled 请求若自带完整体(无 CL),后端立刻回包且响应带 `Connection: close`,后端连接在前端完成映射前被拆掉,响应丢失(follow 只拿到自己的 200)。故第二三段 smuggled GET 也带 `Content-Length: 0` 保持不完整。
- smuggled 字节绕开前端改写:前端加的头必须自己写进 smuggled 头块;只有 follow 请求的转发形态才带前端自己的头。

## 三段链
1. 反射 oracle:POST / 的 search 回显为 `<h1>N search results for '...'</h1>`(GET 不反射)。
   smuggle = `POST / ... Content-Length: 0 \r\n\r\nsearch=`,follow = `GET /`。deficit 40 的 snip 直接给出前端加头 `X-AnopMM-Ip: 216.195.192.85`(头名每实例随机)。
2. `GET /admin` + `X-AnopMM-Ip: 127.0.0.1` + `Content-Length: 0`,deficit 20 → target 3517B 为 admin 面板,含 `/admin/delete?username=carlos`。
3. 同形状打 `/admin/delete?username=carlos` → 302 Location: /admin;banner_verdict 翻牌 `<h4>Congratulations, you solved the lab!</h4>`。

## 复用判据
CL.TE 下要“看到 smuggled 请求的响应”,别追求请求完整;把 CL 留小(体不足)才是读物通道,deficit 20-40 起步。range_launch 直接吃 canonical 路径;page_read 用于题面(解题块被剥离)。
