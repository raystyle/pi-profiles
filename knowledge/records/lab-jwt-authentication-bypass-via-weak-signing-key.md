---
title: "lab-jwt-authentication-bypass-via-weak-signing-key"
---

# lab-jwt-authentication-bypass-via-weak-signing-key

PortSwigger academy, JWT 族。会话 JWT 以 HS256 签名，密钥可猜；目标：伪造 administrator 会话，删 `carlos`。

## 靶面

- `POST /login` 为 JSON 校验且要求页面 `csrf` token：先 GET `/login` 取 `csrf`，再带同 jar POST `csrf`+`username`+`password`，302 的 `Set-Cookie: session=<JWT>` 落 jar。
- JWT 紧凑形式 `header.payload.signature`；签名为 `HMAC-SHA256(secret, "header.payload")` 的 base64url 无填充。
- 校验端按 `kid` 查密钥：伪造时须保留原 `kid`，改动 kid 会破坏密钥查找。

## 攻击链

1. `page_read` 规范 lab 路径取得 widget-lab-id；`range_launch launch-url <path> --jar /tmp/cj1.json` 直接起实例（无需单独取 id）。
2. 登录：GET `/login` 提取 `csrf`，同 jar POST 登录，`session=<JWT>` 落 jar。
3. 离线爆密钥：候选表用公开 wallarm `jwt.secrets.list`（103 764 唯一条目），跑 `jwt_secret crack <token> --wordlist FILE`；命中 `secret1`，第 51 次尝试，耗时不足一秒。
4. 伪造：`jwt_secret forge <token> --secret secret1 --sub administrator --exp 9999999999 --host <instance-host> --jar /tmp/jwt_admin.json`。伪造头保留原 `kid`，仅改 payload 声明后重签。
5. `http_session get /admin --jar /tmp/jwt_admin.json` 返回用户表；`GET /admin/delete?username=carlos` 回 302 至 `/admin`，控制台打印 `User deleted successfully!`。

## 验证

翻牌不体现在 delete 跳：删后重访 `/admin` 读横幅（`academyLabBanner is-solved` 加 `Congratulations, you solved the lab!`）；对基址跑 `banner_verdict` 为同一检查。

## 工具面

`jwt_secret`（项目件，`.pi-rs/rust-scripts/jwt_secret.rs`）覆盖整个离线半：`crack` 沿词表或内联列表穷举 HMAC 密钥、首个命中即停；`forge` 重组头/payload 重签，可选直接写入 `http_session` jar；`selftest` 在合成 token 上跑通两半。
