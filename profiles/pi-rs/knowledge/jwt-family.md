---
title: JWT 族:签名面与算法混淆
---

# JWT 族:签名面与算法混淆

同一类型漏洞:JWT 的签名/校验面被滥用。共性成因是验签侧按令牌自述的
`alg` 选算法;当 RS256 公钥被当作 HS256 的 HMAC 密钥时,攻击者可用公钥自造令牌。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| 算法混淆(公钥已知) | 会话为 RS256;`/jwks.json` 暴露 `n`/`e` 与 `kid` | 由 JWKS 转 SPKI PEM,以其为密钥签 HS256(`sub=administrator`) | JWT-2 |
| 算法混淆(公钥未知) | 无 JWKS;可获取两枚同密钥 RS256 令牌 | 由 `gcd(s1^e − EM1, s2^e − EM2)` 反推模数 `n`(e=65537),再转 PEM 签 HS256 | [[lab-jwt-authentication-bypass-via-algorithm-confusion-with-no-exposed-key]](批 33 solved) |
| kid/头注入(待录) | `kid` 进路径/命令/文件 | 头 `kid` 指向可控文件或注入 | 待录 |

## 共性

1. 头 `alg` 由令牌自述:验签侧“按 alg 选算法”即混淆根因;`kid` 仅提示密钥选择。
2. 算法混淆只需“公钥字节”:RSA 公钥 PEM(SPKI)既是公钥,也被当作 HMAC 密钥。
3. 公钥两条来源:直接暴露(`/jwks.json` → `n`,`e`)或由两枚签名反推。

## 判定与收尾要点

- 判定锚点:伪造令牌实际进入 `/admin` 并删 `carlos`;仅签名被接受不算解。
- 令牌经 `Cookie: session=<jwt>` 回放;`sub` 改目标身份,`exp` 置远期。
- 反推模数:`EM` 用 EMSA-PKCS1-v1_5(SHA-256,块长=签名长度);
  `s^e − EM` 两两取 `gcd` 得 `n`(通常即为 `n`)。
- 收束:`solved_check` 读实例横幅。

## 工具面

- `jwt` 件:`decode`(拆头/载荷)、`pubkey`(JWKS→SPKI PEM)、`rsa-from-tokens`(两令牌反推模数→PEM)、
  `forge`(JWKS 直签 HS256)、`sign --pem`(PEM 直签 HS256)。
- `lab_http`(登录取令牌/回放 Cookie)、`lab_page`、`lab_launch`、`solved_check`、`json_pick`。

## 反推实现

- `jwt rsa-from-tokens` 已改 **GMP 快速路径**:Rust 只算 `(s_i, EM_i)`,内置 C 源用 `gcc -O2 -lgmp` 按需编译到
  `$TMPDIR/pi-rs-rsa2n/rsa2n` 做 `mpz_pow_ui` + `mpz_gcd`(22~35s);`num-bigint` 纯 Rust 仅作无 gcc/gmp 时的回退。
  本机已备 `gcc` + `libgmp.so`(无 gmpy2 → 不走 python,纪律上也禁)。
- **阶数伪影**:两个 residual 可能同含小因子,`gcd` 会比真 `n` 多出 `2` 等(实测 2049 bit = `2n`)。
  RSA 模数必为奇数且两因子极大,故对 gcd 做 `strip_small_factors`(试除 2..=65536)安全且必要。
- PEM 用 `spki_pem`(BEGIN PUBLIC KEY、64 列、尾换行)当 HMAC 密钥即可;`kid` 沿用原令牌头的值。

## 实录溯源

- [[lab-jwt-authentication-bypass-via-algorithm-confusion]]、[[lab-jwt-authentication-bypass-via-algorithm-confusion-with-no-exposed-key]](无 JWKS 公钥反推,算力未收口)

## 相关族

- 方法论:web-vuln-methods(seed 层,按名引用)。
