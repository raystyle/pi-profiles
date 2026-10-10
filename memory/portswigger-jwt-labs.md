---
metadata:
  node_type: memory
name: "PortSwigger JWT Labs"
description: "lab-jwt-authentication-bypass-via-jwk-header-injection(arm A)已解:jwk 内嵌公钥须与头部 kid 一致,新件 jwk_inject.rs 一键签发并写 jar"
last_updated: 2026-10-10T12:22:35+08:00
created: 2026-10-10T12:22:35+08:00
---

- 路径 `/web-security/jwt/lab-jwt-authentication-bypass-via-jwk-header-injection` 直接 page_read 取 widget-lab-id;`range_launch launch-url <该路径> --jar /tmp/cj1.json` 一次起实例(reused:false),无需另调取号。
- 登录腿:`http_session submitform /tmp/login.html` 在该 lab 返 400 "Missing parameter"(表单 input 是 `type=username`/`type=password` 且无 value);改 `http_session post /login --form csrf=<页面值> --form username=wiener --form password=peter` 成功(302 `/my-account?id=wiener`),响应 Set-Cookie 即 RS256 会话 JWT。
- 新件 `.pi-rs/rust-scripts/jwk_inject.rs`(rsa 0.9 + sha2 0.10 带 `oid` feature + rand 0.8):自持 RSA 2048,公钥以 `jwk` 内嵌 JWT 头,RS256 签 `sub=administrator`,token 写入 http_session 兼容 jar(host->cookie 名->值),`--selftest` 本地复验签名与 jar 形状。
- 判读律(本题关节点):头部有 kid、但 jwk 对象内无 kid → 服务端弃会话(302 /login 且 Set-Cookie 清 session,即"token 被拒");**jwk.kid 与头部 kid 一致,或头部完全不带 kid → 200 且 `Your username is: administrator`**。故该实现是"按 kid 对齐内嵌密钥"而非纯 jwk 优先,件默认已按此形状签发。
- 利用腿:`GET /admin`(200,Users 列表含 carlos Delete 链接)→ `GET /admin/delete?username=carlos`(302 `/admin`)→ `banner_verdict <root> --jar` 回 `solved:true`。
- 技巧点:http_session 会跟随响应把 jar 里的 session 清成空(`Set-Cookie: session=;`),后续请求 `sent_cookie` 为 null 时结论无效 —— 判"服务端是否接受 token"必须重注入 token 再请求,并核对 hop 的 sent_cookie。
- 编译坑:sha2 0.10 需 `features=["oid"]`,否则 `SigningKey::<Sha256>::new`/`VerifyingKey::<Sha256>::new` 因 `AssociatedOid` 不满足不编译;`json!` 里 9999999999 必须写 `9999999999i64`。
