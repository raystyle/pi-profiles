---
title: lab-javascript-url-some-characters-blocked
---

# lab-javascript-url-some-characters-blocked

> evidences: [[xss-context-family]]

- 题面:反射 XSS,调 `alert` 且消息含 `1337`。
- 实例(批48):https://0a0e000d03cef0cc8025b21300b100b2.web-security-academy.net(本 lab 无 exploit server)
- 状态:**stuck**,但**在档核心前提被推翻**(旧结论作废)

## 推翻的旧结论(合成点击假阴性)

旧档写"现代 Chrome 不对 `javascript:` URL 做百分号解码(合成 `a.click()` 实测不执行)"⇒ **假阴性**。批48 用**真用户手势**(`page_alert --click`,CDP Input 管线)实测:

```
data:text/html,<a id=t href="javascript:alert%281337%29">go</a>  --click #t
→ {"fired":true,"alerts":["alert:1337"]}
```

⇒ **真点击下 Chrome 会先百分号解码 javascript: URL 再执行**。合成 `el.click()` 无用户手势,Chrome 静默不导航,不能用来判否定。旧档由此推出的"双重编码换字面括弧"整条推理作废。

## 编码器(新实例复核,逐槽一致)

保留:alnum 与 `" { } | ^ $ , * / ~ _ - . !`(空格→`+`);百分号编码:`'`→%27 `;`→%3b `:`→%3a `=`→%3d `@`→%40 `<`→%3c `>`→%3e `&`→%26 `+`→%2b `?`→%3f;**删除 `( ) [ ]` 反引号 反斜杠与所有 `%`**。

- `x=alert(1337)` → `x=alert1337`(括弧整体消失);
- **参数名**里的括弧同样被删(`&(()=b` → `&%3db`);
- **路径**里的括弧 → 404(该页无反射面)。

⇒ `(` `)` 在任何注入槽都不可达;`%` 不可达 ⇒ 无法用 `%28` 借浏览器解码造括弧。

## 现在的墙(比旧档更清楚)

真点击解码给了我们 `'`(%27 在点击时变 `'`)⇒ **JS 单引号串可破**。但模板是:

```
fetch('/analytics', {method:'post',body:'/post?postId=1&x=<注入>'}).finally(_ => window.location = '/')
```

注入只能改 `body` 字符串(及 options 对象内部);**作者侧写不出任何括弧**,因此 `alert(1337)` 只能复用模板里两处固定调用点(`fetch(`…`)`、`.finally(`…`)`)的括弧 —— 这两处的被调者是固定文本(`fetch` / `.finally`),构造尚未完成。无括弧可用的调用语法(模板字面量、`new`、`throw`)也都被字母表封死(反引号被删)。

## 未决面/下一步

- 用 `page_eval_batch`/`browser_suite eval` 在**同一模板**上枚举"破串后如何借用既有括弧"的构造(奇偶引号配对 + `;` 截断 + `//` 注释尾段);`;` `:` `=` `?` `@` `&` `<` `>` `+` 都可用,先在小样本上做语法合法性二分。
- 判读纪律:**凡涉及 javascript: URL 的执行判定,一律用真输入管线**(page_alert/browser_suite 的真实点击),禁止用合成 click 下结论。

## 复现命令

```
range_launch launch C29F5F30…6711A345 --jar ~/.pi-rs/agent/chrome-jar.json
page_alert 'data:text/html,<a id=t href="javascript:alert%281337%29">go</a>' --click '#t' --settle-ms 2500
http_session get "https://<inst>/post?postId=1&x=alert(1337)" --jar <jar> --out /tmp/l.html   # 看编码器
```

## 关系

- 族:[[xss-context-family]](该族的 javascript: 行按本档改写:真手势下会解码)。
