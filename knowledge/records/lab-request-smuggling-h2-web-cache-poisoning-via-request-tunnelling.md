---
title: "lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling"
links:
  - target: h2-smuggling-family
    relation: evidences
---

# lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling

> evidences: [[h2-smuggling-family]]

PortSwigger `request-smuggling/advanced/request-tunnelling/lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling`。
实例 `https://0a0a00b404254c5281c44d3600330085.web-security-academy.net/`(重开实例 `0a0800380482105880c4262200bb0032`)。

- 判定:**stuck / unresolved**(隧道机制已实测成立,缺“把 alert(1) 塞进被缓存响应体”的那一步)。

## 已证实(件 h2_req v1.1.0,`--path` 现在也展开 `\r\n` 转义)

1. **注入点是 `:path`,不是 header**:`0a0a…` 实例上 header 值/名内的 CRLF 都被前端 RST_STREAM(`{"rst_stream":1}`),
   `0a08…` 实例上 header **NAME** 内 CRLF 会被透传(`--hdr2`);两实例的 `:path` 都原样进 h1 请求行 →
   可注入整条第二请求(第二请求的 request line 尾会被前端追加的 ` HTTP/1.1` 吞掉,故末行用 `X: ` 之类的哑头吸收),
   且 tunnelled 请求**确实执行**。
2. **非盲 HEAD 过读**:外层用 `HEAD <path>`(CL = 该资源 GET 体长),注入的第二请求响应会被当作 body 读出来:
   ```
   h2_req <inst> --method HEAD --path '/post?postId=1 HTTP/1.1\r\nHost: <HOST>\r\n\r\nGET /post?postId=1 HTTP/1.1\r\nHost: <HOST>\r\nX: '
   ```
   → 本端 h2 body 里出现嵌套的 `HTTP/1.1 200 OK\r\n…Content-Length: 8954\r\n\r\n<!DOCTYPE html>…`(整段 h1 响应)。
   若隧穿响应 < 外层 CL,前端会一直等字节(实测挂住);CL 未读满时前端回 `500 Received only N of expected M bytes`,
   可据此反推长度 → 证明“CL 从一条响应、body 从另一条”混合。
3. 缓存:`/`、`/analytics`、404 全部 `Cache-Control: max-age=30` + `X-Cache: miss/hit`,键含 query
   (`/?ka=1` 与 `/?kb=2` 各自 miss);`GET /` 复核为 `X-Cache: hit`、`Age: 7`(受害者每 15s 开首页)。
   所有探测头(XFH/X-Host/Ua/……25 个)不反射且不参与缓存键。

## 未决面(为什么没翻牌)

- 该 app 的**所有**可见反射都是 HTML 转义(搜索页 H1、评论 name/comment/website、confirmation 的 postId href、
  `POST /post/comment` 的 `"Invalid email address: <html-escaped>"` JSON 回显)→ 无法把裸 `<script>` 送进缓存。
- 已验证可用的原语:①`:path` 注入隧道;②`HEAD` 过读;③被缓存 URL 必须是 `HEAD /` 或 `GET /`(query 入键)。
- 未解的一环:找出一个“回显未转义输入”的响应(理论页说这类 app 常有 JSON 回显端点),让它的字节被过读进 `/` 的缓存条目。
  本实例候选端点只有 `/`、`/post?postId=N`、`/post/comment`(+confirmation)、`/image/*`、`/resources/*`、`/analytics`
  (恒 200/CL 0),穷举 wordlist 无其它路由。
- 候选交付:用 Host 派生的绝对 302(`GET /resources` + `Host: exploit-…` → `Location: https://exploit-…/resources/`)
  配合非盲过读,把 302/脚本字节混进被缓存的外层响应;但缓存键含 query、`GET /` 外层响应 CL 固定,
  如何让外层 body 携带嵌套字节仍未解。

## 复现命令

```
h2_req <inst> --method HEAD --path '/post?postId=1 HTTP/1.1\r\nHost: <HOST>\r\n\r\nGET /post?postId=1 HTTP/1.1\r\nHost: <HOST>\r\nX: ' --read-ms 3000
```
