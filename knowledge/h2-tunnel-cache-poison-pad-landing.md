---
title: h2 隧道毒缓存的垫片落点与验收(h2 tunnel cache poisoning)
---

# h2 隧道毒缓存的垫片落点与验收(h2 tunnel cache poisoning)

用 h2 请求隧道把一个「反射载荷的响应」记进目标 URL 的缓存。批50 在
[[lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling]] 上一次收口,
方法可照搬。

## 形状

外层用**无 body 的方法**(`HEAD <url>`);`:path` = `<url> HTTP/1.1\r\nHost: <host>\r\n\r\n<内层请求>`
再垫到目标长。前端降级成 h1 时,后端对 `<url>` 只回头,前端按自己的**期望长度 M**继续读字节,
把随即到达的**内层响应原文字节**当成 `<url>` 的体并缓存(= [[h2-tunnelling-and-h2cl-practice]]
的记账 oracle 的另一面)。

## 三条硬规则

1. **M 活测,不写死**:`GET <url>` 的 `content-length` 就是 M(实例间会变;批49 8419 → 批50 8566)。
2. **垫片必须落内层 URI 内(query 段)**,且内层响应要 **≥ M**。嵌套 302 的 `Location` 逐字回显
   内层 URI ⇒ 垫 1 字节长 1 字节;`h2_req --pad-path-to N` 把整条 `:path` 截到 N,故让 `:path`
   **以内层 URI 结尾**(别写内层 ` HTTP/1.1\r\nHost:`)⇒ 垫片自动落进 query。pad 9000 → Location ≈8.7KB > 8566 即通。
3. **`:path` 不得超 HEADERS 帧上限 16384B**:超了前端 GOAWAY(无响应,易误读成"没注入")。

## 验收与判读纪律

- 注入回执的读数 = **200 + `x-cache: miss` + `content-length == M`**,body 即嵌套响应原文。
- 随后 `GET <url>` → **`x-cache: hit`** 且 body 同上 ⇒ 毒已入键。
- 载荷可以是**裸 `<script>` 藏在 302 的 Location 里**:cached 应答 content-type 仍是 `text/html`,
  浏览器按 HTML 解析即执行,不必把毒伪装成 HTML/JS 页。
- **毒在缓存时 `banner_verdict` 读不到 `is-solved`**(它 GET `<url>` 拿到的是毒 body);
  改读另一页 lab header(任何未缓存页)判收口。
- 多格扫描件(如 `tunnel_variant_scan`)的 `clean` 直发格会**先写缓存**,使后续注入格全成 HIT ⇒
  假阴;本例改用单发 + 自算 M。短读报错口径按 lab 不同(本例 `500 Communication timed out`,
  ACL 题是 `500 Received only N of expected M`)。

## 关系

族 [[h2-smuggling-family]]、[[cache-poisoning-family]];方法 [[h2-tunnelling-and-h2cl-practice]]、[[cache-and-smuggling-live-mechanisms]]。

## Links

- evidences: [[h2-smuggling-family]]

- solves: [[lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling]]
