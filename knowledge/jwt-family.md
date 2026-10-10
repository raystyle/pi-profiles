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
| 未验签 | 服务端不验签 | 改载荷不换签名即过:先试直改,再试空签名 | 批 jwt 8/8 |
| alg:none 变体面 | 服务端接受 `none` 算法 | `alg` none/None/nOnE 枚举 × 尾点保留/丢弃 × typ 保留/丢弃的变体阵;件 `jwt_none` 一次出全阵+逐 jar | 批 jwt 8/8 |
| 弱密钥(HS256) | HS256 且 HMAC 密钥弱 | 离线爆破(公开 jwt.secrets 词表);forge 保原 `kid`——验签按 `kid` 取钥,换 `kid` 即断查 | [[lab-jwt-authentication-bypass-via-weak-signing-key]] |
| jwk 头注入 | 验签钥由令牌头 `jwk` 给出 | 头内嵌自签 `jwk`,服务端信任 token 自带验签密钥 | 批 jwt 8/8 |
| jku 头注入 | 验签钥由令牌头 `jku` URL 给出 | 头 `jku` 指向攻击者托管 JWKS,服务端不校验域名 | 批 jwt 8/8 |
| kid 路径遍历 | `kid` 作本地存储索引取钥 | `kid` 路径遍历至已知内容文件(`/dev/null` 即空 HMAC 密钥) | 批 jwt 8/8 |

## 共性

1. 头 `alg` 由令牌自述:验签侧“按 alg 选算法”即混淆根因;`kid` 仅提示密钥选择。
2. 算法混淆只需“公钥字节”:RSA 公钥 PEM(SPKI)既是公钥,也被当作 HMAC 密钥。
3. 公钥两条来源:直接暴露(`/jwks.json` → `n`,`e`)或由两枚签名反推。
4. 三头注入同为「验签钥由令牌自述指针解析」——`jwk`=值内嵌、`jku`=URL 间接、`kid`=本地索引。

## 判定与收尾要点

- 判定锚点:伪造令牌实际进入 `/admin` 并删 `carlos`;仅签名被接受不算解。
- 令牌经 `Cookie: session=<jwt>` 回放;`sub` 改目标身份,`exp` 置远期。
- 反推模数:`EM` 用 EMSA-PKCS1-v1_5(SHA-256,块长=签名长度);
  `s^e − EM` 两两取 `gcd` 得 `n`(通常即为 `n`)。
- 收束:`solved_check` 读实例横幅。

## 工具面

- `jwt` 件:`decode`(拆头/载荷)、`pubkey`(JWKS→SPKI PEM)、`rsa-from-tokens`(两令牌反推模数→PEM)、
  `forge`(JWKS 直签 HS256)、`sign --pem`(PEM 直签 HS256)。
- `jwt_none`:alg:none 变体阵(`alg` 大小写枚举 × 尾点 × `typ`),一次出全阵+逐 jar。
- `jwk_inject`:头密钥注入签发,`jwk` 内嵌 / `--jku` 托管 JWKS 两形,`--selftest` 本地复验。
- `jwt_hs_forge`:任意字节源 HMAC 重签,多 `kid` 逐 jar,`/dev/null` 空密钥。
- `jwt_secret`:HS256 crack+forge,selftest。
- `lab_http`(登录取令牌/回放 Cookie)、`lab_page`、`lab_launch`、`solved_check`、`json_pick`。

## 反推实现

- `jwt rsa-from-tokens` 已改 **GMP 快速路径**:Rust 只算 `(s_i, EM_i)`,内置 C 源用 `gcc -O2 -lgmp` 按需编译到
  `$TMPDIR/pi-rs-rsa2n/rsa2n` 做 `mpz_pow_ui` + `mpz_gcd`(22~35s);`num-bigint` 纯 Rust 仅作无 gcc/gmp 时的回退。
  本机已备 `gcc` + `libgmp.so`(无 gmpy2 → 不走 python,纪律上也禁)。
- **阶数伪影**:两个 residual 可能同含小因子,`gcd` 会比真 `n` 多出 `2` 等(实测 2049 bit = `2n`)。
  RSA 模数必为奇数且两因子极大,故对 gcd 做 `strip_small_factors`(试除 2..=65536)安全且必要。
- PEM 用 `spki_pem`(BEGIN PUBLIC KEY、64 列、尾换行)当 HMAC 密钥即可;`kid` 沿用原令牌头的值。

## 族地板带

- 8-18 冒烟带、中位 15;头注入面 12-16;铸件题 36 离群(件迭代成本)。

## 实录溯源

- [[lab-jwt-authentication-bypass-via-algorithm-confusion]]、[[lab-jwt-authentication-bypass-via-algorithm-confusion-with-no-exposed-key]](无 JWKS 公钥反推,算力未收口)、[[lab-jwt-authentication-bypass-via-weak-signing-key]](HS256 弱密钥离线爆破)

## 相关族

- 方法论:web-vuln-methods(seed 层,按名引用)。
