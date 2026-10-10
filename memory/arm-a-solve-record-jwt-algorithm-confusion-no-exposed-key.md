---
metadata:
  node_type: memory
name: "Arm A Solve Record - JWT Algorithm Confusion No Exposed Key"
description: "Arm A: JWT algorithm-confusion with no exposed key solved — rsa-from-tokens 取公钥, HS256 混淆签 administrator, /admin 删 carlos, banner solved"
last_updated: 2026-10-10T12:17:40+08:00
created: 2026-10-10T12:17:40+08:00
---

## 2026-06-05 lab-jwt-authentication-bypass-via-algorithm-confusion-with-no-exposed-key (arm A, solved)

- Path: /web-security/jwt/algorithm-confusion/lab-jwt-authentication-bypass-via-algorithm-confusion-with-no-exposed-key (page_read 直取 lab_id E63829C5..., range_launch launch-url, reused:false, jar /tmp/cj1.json)
- Instance 0aaa0008042d27d480a735af00a30023; login POST /login 需 csrf(get /login 页面取), 两 jar 各登一次 wiener:peter 得两枚 RS256 token(仅 exp 不同, kid 相同)
- 无暴露 key 的取 key: jwt rsa-from-tokens T1 T2 --out /tmp/pub.pem, 21.7s 出 2048 位 SPKI PEM(gcd 法)
- 混淆: jwt sign /tmp/pub.pem administrator --kid <原 kid> --out /tmp/admin.jwt → HS256 token 以 PEM 字节为 HMAC 密钥
- jar 手写 {host:{session:token}}(http_session 原生格式), GET /admin 直接 200 出 Users 列表(审计视角 = administrator)
- GET /admin/delete?username=carlos → 302 /admin; banner_verdict 首页 = solved:true
- 件序列: page_read → range_launch → http_session(get/登录 x2) → jwt(rsa-from-tokens, sign) → http_session(/admin, delete) → banner_verdict
- 要点: 路由/身份校验只用 sub; kid 保留原值即可; 整链 5 个件、无手搓脚本

