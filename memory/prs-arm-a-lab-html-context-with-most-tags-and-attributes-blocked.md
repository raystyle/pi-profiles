---
metadata:
  node_type: memory
name: "PRS arm A lab-html-context-with-most-tags-and-attributes-blocked"
description: "arm A 基线:lab-html-context-with-most-tags-and-attributes-blocked 一次通过 - 标签面盘仅 body/math/text/foreignObject/custom 放行,属性面盘 onresize 放行,载荷 `<body onresize=print()>` 经 exploit server iframe 缩宽自动触发,banner solved"
last_updated: 2026-10-08T23:55:01+08:00
created: 2026-10-08T23:55:01+08:00
---

arm A 基线:lab-html-context-with-most-tags-and-attributes-blocked(反射 XSS,HTML 上下文,WAF 封大部分标签与属性)冷实例一次通过。

- 实例:page_read 取 widget-lab-id → range_launch(jar /tmp/cj1.json,reused:false)→ 0a34000903cd484c80ca49c400020032。
- 反射点:`<h1>0 search results for '<marker>'</h1>`,payload 原样落地(无编码)。
- 标签面盘:raw_matrix 一次扫 139 个标签名,仅 body / math / text / foreignObject / custom / foo / xss 返回 200,其余(含 a、img、svg、script、h1)全 400。
- 属性面盘:raw_matrix 一次扫 118 个属性,放行 onresize、onbeforetoggle、onstorage、ononline、onoffline、onratechange、onstalled、onsuspend、onabort、onemptied、onpointercancel、onslotchange、onbeforeinput、onformdata 与 autofocus/tabindex/style/accesskey/id/class/src/href/formaction/contenteditable/popover/is;onload、onfocus、onerror、onclick 等一律 400。
- 载荷:`<body onresize=print()>`,GET /?search=%3Cbody%20onresize%3Dprint()%3E → 200 且原样反射(注入的 body 起始标签按 HTML 解析规则把属性并入已存在的 body 元素)。
- 投递(自动触发,零交互):exploit server 存 `responseFile=/exploit` + body `<iframe src="https://<lab>/?search=%3Cbody%20onresize%3Dprint()%3E" onload=this.style.width='100px'></iframe>`,再 formAction=DELIVER_TO_VICTIM;iframe 缩宽触发 resize 事件 → body 的 onresize 调 print()。
- 判据:banner_verdict → solved_class true,congrats_line "Congratulations, you solved the lab!"。
- 件面:raw_matrix(一次一封的标签/属性面盘)、http_session(反射确认 + exploit server 存/投两发,form 体手工 urlencode,内层 %3C 写成 %253C)、nap、banner_verdict;exploit server 无需登录,POST / 即可 STORE / DELIVER_TO_VICTIM。
- 坑:http_session 信封剥 script 块,反射槽显示为空不等于未反射,以 body_snippet/h1 行为准;raw_matrix 每变体一条新 TLS 连接,~1.4s/条,139 条约 3.5 分钟,给足 timeout。

