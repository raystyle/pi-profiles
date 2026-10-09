---
metadata:
  node_type: memory
name: "PRS arm A lab-onclick-event-angle-brackets-double-quotes-html-encoded-single-quotes-backslash-escaped"
description: "arm A 基线:lab-onclick-...(存储 XSS 入 onclick)冷实例一次通过 - 网站字段用 &apos; 破 JS 单引号串,page_alert fired + banner solved"
last_updated: 2026-10-09T07:06:43+08:00
created: 2026-10-09T07:06:43+08:00
---

## 2026-10-09 冷实例一次通过(reused:false)

题:Stored XSS into onclick event(尖括号与双引号 HTML 编码、单引号与反斜杠转义),实例
https://0a24002003bb70cb864425d500fe0043.web-security-academy.net/,widget-lab-id
53ddf9ef1abeee631133a34645f234b5a0115eb9e739c4cebbe2a4668735893b。

路径:`range_launch launch-url /web-security/.../lab-onclick-...`(--jar /tmp/cj1.json)直接起实例。

1) 注入面定位:首页 → `/post?postId=1` 评论表单,字段 csrf/postId/comment/name/email/website;
   先投一条无害评论(website=http://p1.invalid/z)读回模板:
   `<a id="author" href="URL" onclick="var tracker={track(){}};tracker.track('URL');">name</a>`
   —— website 值同时落 href 双引号属性与 onclick 内 JS 单引号串。
2) 编码面实测:服务端 HTML 编码 `<` `"`,给 `'` 加反斜杠,但**不动 `&`** ⇒ `&apos;` 原样落库。
3) 载荷(website 字段):`http://foo?&apos;-alert(1)-&apos;`
   落库形态:`tracker.track('http://foo?'-alert(1)-'');`(实体在属性解析期解码成真引号)⇒ 破串即执行。
4) 判定:POST /post/comment 302 → 确认页横幅立刻 `is-solved`;banner_verdict(post 页) solved:true +
   "Congratulations, you solved the lab!";page_alert(直接调用该评论 `a.onclick` 函数体)alerts=["alert:1"] fired=true。

两个实测坑(通用):
- 多评论时 `id="author"` 重复,`--click '#author'`/querySelector 只命中第一条(无害评论)⇒ 必须按 textContent 定位。
- page_alert 用 `--driver "...click()"` 点该链接时未捕获弹窗(点击触发导航,求值上下文先销毁);
  改为直接 `f=a.onclick; f.call(a,{})` 调用解析后的处理器即 fired=true。

件链:range_launch → http_dump(读模板)→ http_session post(投载荷)→ banner_verdict + page_alert。

