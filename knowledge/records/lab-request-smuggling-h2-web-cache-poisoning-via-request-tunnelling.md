---
title: lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling
---

# lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling

> evidences: [[h2-smuggling-family]]

PortSwigger `request-smuggling/advanced/request-tunnelling/lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling`。批 44 实例 `0aa900a404a4a17c80ee3a51003700d4`。目标:毒 `/` 的缓存,使每 15s 访问首页的受害者浏览器执行 `alert(1)`。题面明确:**前端不复用后端连接** ⇒ 只能 tunnelling。

- 判定:**stuck**(载荷出处已实测确证;堵在"把嵌套字节读够外层长度"的算术与请求行长度限制)。

## 批 44 新增证据

1. **载荷出处确证(未编码回显)**:`GET /resources/labheader/js?<script>alert(1)</script>` ⇒
   `302 Location: /resources/labheader/js/?<script>alert(1)</script>`(**原样、不编码**)且响应带 `Cache-Control: max-age=30`、`X-Cache: miss` ⇒ 该 URL 自身可缓存。
2. **`/` 的缓存键要求**:受害者只访问 `/`,故毒必须记在 `/` 上 ⇒ 外层请求必须是 `HEAD /`(前端"预期长度"= `/` 的体长 8640~8880),由后端**过读**把嵌套响应字节当成 `/` 的体。
3. **垫片请求不可行**:嵌套重定向的响应带 `Keep-Alive: timeout=0`(后端随即关连接),所以"再注入一条 `GET /` 凑字节"会让前端永远等不满 ⇒ 实测无任何响应(`received_bytes: 0`,前端挂住)。
   ⇒ 唯一的嵌套响应(那条 302)**自身**必须 ≥ 外层期望长度 ⇒ 查询串要垫到 ~8.4KB。
4. 批 41 的可读通道仍在:`:path` 内 CRLF 透传 + 外层 HEAD 过读,嵌套响应原始字节会出现在 h2 body 里。

## 未决面

- 需要一次"垫到 ~8.4KB 的 h2 `:path`"注入;未知量:前端 h1 请求行上限(8KB 常见默认,超限 414/400)、以及本实例 `/` 的精确体长(须现测)。
- 缺件:能让 h2 注入体**由文件/生成器提供**(而不是手写进 argv)的件,否则 8.4K 字符无法可靠落进一条命令。

## 复现

```
h2_req <inst> --method GET --path '/resources/labheader/js?<script>alert(1)</script>'   # 载荷出处
h2_req <inst> --method HEAD --path '/ HTTP/1.1\r\nTeto: teto\r\n\r\nGET /resources/labheader/js?<script>alert(1)</script><PAD> HTTP/1.1\r\nHost: <inst>\r\nTeto: teto'   # 过读+垫片形
```
