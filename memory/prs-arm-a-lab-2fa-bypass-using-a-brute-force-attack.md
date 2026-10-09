---
metadata:
  node_type: memory
name: "PRS arm A lab-2fa-bypass-using-a-brute-force-attack"
description: "arm A 基线:lab-2fa-bypass-using-a-brute-force-attack 冷实例解出 - h2 多路复用重登录扫码,第 2 轮 0600 命中,banner solved"
last_updated: 2026-10-09T09:22:55+08:00
created: 2026-10-09T09:22:55+08:00
---

### 终态
solved(congrats):carlos 的 2FA 码 0600 被枚举出,mfa_post 302 → /my-account?id=carlos,banner solved=true。

### 起点与实例
- 题页 `/web-security/authentication/multi-factor/lab-2fa-bypass-using-a-brute-force-attack`(canonical 无误),widget-lab-id `28468f8ce00fc3bd7c323d598659c718d294c4aa39e6a5e2b244b48b1116493e`。
- `range_launch launch-url <路径> --jar /tmp/cj1.json` → reused:false,实例 `0a3c00aa03b30b068275242500fd00ec`;题面只给 `carlos:montoya` 与「码会重置、可能要重跑几轮」。

### 机制(实测)
- 码为 4 位;每次猜码必须重走 GET /login(取 csrf) → POST /login(首因子,302) → GET /login2(取 csrf) → POST /login2(mfa-code),错码后会话被弃,故 4 请求/次。
- **第一轮全量 0-9999(=10000 次)零命中**;第二轮从 0 起重扫,第 657 次(码 0600)命中 ⇒ 码在扫描期间会重置(每次登录重生成或短 TTL 二者皆与观测相容),单轮全量扫描可能空过,必须多轮。
- 命中判据:mfa_post 302 + Location `/my-account?id=carlos`;胜出会话写回 jar,再 GET /my-account 200 复核。

### 件选型(吞吐差 20 倍)
- `mfa_relogin_brute`(HTTP/1.1, ureq 每请求新建连接):4 线程 3 req/s、32 线程 15 req/s 且出现 8% status=0 传输错误 ⇒ 10000 次约 45 min,不划算。
- `h2_relogin_brute --streams 32`(单条 h2 多路复用,ALPN h2):**65 req/s(≈16.5 次/s),4 步直方图 2500/2500/2500/2500 全净、零传输错误**,全量一轮 ~10 min;`--streams 64` 未见提速(亦 65 req/s),32 即够。重登录型 2FA 枚举一律走 h2 腿。

### 过程坑
- 一次 `rs_execute` 返回 `error: No such file or directory (os error 2)` exit 1、无信封,原样重跑即恢复(执行器瞬断,非件缺陷)。
- 命中轮 confirm 用的 `--confirm-path /my-account`(不带 ?id=carlos)在 carlos 会话下同样 200,足够作复核。

