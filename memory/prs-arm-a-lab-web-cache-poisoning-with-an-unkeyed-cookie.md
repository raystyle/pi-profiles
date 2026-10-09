---
metadata:
  node_type: memory
name: "PRS arm A lab-web-cache-poisoning-with-an-unkeyed-cookie"
description: "arm A 基线:lab-web-cache-poisoning-with-an-unkeyed-cookie 冷实例一次通过 - fehost cookie 反射进页内 JSON 串且不入 cache key,`x\"}%3Balert(1)%3B//` 投毒首页条目后 victim 命中翻牌"
last_updated: 2026-10-09T11:13:18+08:00
created: 2026-10-09T11:13:18+08:00
---

arm A 基线:lab-web-cache-poisoning-with-an-unkeyed-cookie 冷实例(reused:false)一次通过。

路径与实例
- launch-url 直吃 canonical academy 路径即得实例(无需 page_read 取号)。
- 站点:首页注入 data = {"host":..,"path":..,"frontend":"prod-cache-01"}。

面盘
- cookie `fehost` 的值原样拼进该 JSON 串的 frontend 字段(cookie 名以 marker 探针确认:fehost=ZZZPROBE → "frontend":"ZZZPROBE")。
- 缓存:Cache-Control: max-age=30,响应带 X-Cache/Age;cache key 不含 cookie ⇒ 未键控 cookie。
- 服务端自置 fehost=prod-cache-01;攻击请求显式带 fehost 即覆盖反射值。

两个坑
1. 裸 `;` 在 Cookie 头里被当成 cookie 分隔符,服务端只收到 `x"}`(反射成 `"frontend":"x"}"}`),语句分隔符丢失;必须把 `;` 百分号编码:服务端会解码。
2. 首页条目已 fresh 时投毒请求只会 HIT(不回源);须等 TTL 过期再投,或用独立 cache key 先验反射。

载荷与判定
- 载荷:`Cookie: fehost=x"}%3Balert(1)%3B//` → `data = {...};alert(1);//"}`。
- 投毒请求回 x-cache:miss/age:0;随后无 cookie 客户端取回 x-cache:hit 且 body 仍含 payload ⇒ 缓存投毒证成。
- TTL 只有 30s,用 poison_loop(同一请求,--interval-secs 8)跨 TTL 续投;约 25s 后 banner_verdict 回 solved:true + congrats 行。
- 交册:一次 launch + 4 次探针 + 1 次投毒 + 1 次复核 + poison_loop 续投即收;无新件需求。

