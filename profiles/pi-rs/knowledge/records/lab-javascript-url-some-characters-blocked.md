---
title: "lab-javascript-url-some-characters-blocked"
links:
  - target: xss-context-family
    relation: evidences
---

# lab-javascript-url-some-characters-blocked

> evidences: [[xss-context-family]]

- 题面:Reflected XSS in a JavaScript URL with some characters blocked(/web-security/cross-site-scripting/contexts/lab-javascript-url-some-characters-blocked)
- 实例:https://0aed00d403c1c8ac8151a7f5006a0011.web-security-academy.net
- 判定目标:反射 XSS 触发 `alert('1337' 含 1337)`;状态:**unresolved**(机制未收口)

## 已确认

1. 射点:`GET /post?postId=<int>`(非整数 → 400 "Invalid blog post ID");额外的 query 参数
   整串进入 "Back to Blog" 链接的 JS URL:
   `<a href="javascript:fetch('/analytics', {method:'post',body:'/post?postId=1&x=<URI>'}).
   finally(_ => window.location = '/')">Back to Blog</a>`(URI 为请求 URI,经实例编码器处理)。
2. **实例编码器**(用 `raw_http --request-line` 逐字符实测,输入 `A!'"();{}[]<>&%41+*/,:;=@~^|$`):
   - 百分号编码:`'`→`%27`、`;`→`%3b`、`:`→`%3a`、`=`→`%3d`、`@`→`%40`、`<`→`%3c`、
     `>`→`%3e`、`&`→`%26`(`%41`→`A`:先解码一次再编码);
   - **删除**:`(` `)` `[` `]` 反引号 `\` 与裸 `%`;
   - **保留**:`!` `"` `{` `}` `|` `$` `^` `+` `*` `,` `/` `~` `_` `-` `.` 及空白(空格→`+`)。
3. 现代 Chrome **不对 `javascript:` URL 做百分号解码**(实测 `href="javascript:alert%281337%29"`
   静态属性 click 与 JS 赋值 click 皆不执行;`%27` 亦然)→ `%27` 当不了 `'`。
4. 因此:JS 串是单引号,唯一破串字符 `'` 被编码 → **JS 串不可破**;
   HTML 属性是双引号且 `"` 保留 → 可破出属性,但 `=`/`<`/`>` 均被编码,
   既写不出事件处理器(`onmouseover=…`)也写不出新标签(`<script>`),破出后只能塞属性名。
   固定后缀 `'}).finally(_ => window.location = '/')` 的 `=`/`(`/`)` 无法与注入的属性名对齐。
5. **双重编码也封死**(另一实例 https://0ae200500421c697809d94eb00b00097.web-security-academy.net):`x=%27` 与 `x=%2527` 的响应体长度 9213 vs **9212**(`%2527` 5 字符输入最后只落 2 字符)⇒ 实例管道是**先解码一次(`%25`→`%`)再删裸 `%`**,双重编码换不来真引号;`%2528%2529` 同理(9214,数字残留)。

## 未决面 / 卡点

- 自建编码器模型下,两个实例的两条 XSS 通路(JS 单引号串 / 双引号属性)都被封死;但题面保证可解。
  疑点:实例前置(AWS 边缘?)可能做了与 app 不同的 URI 归一化,或存在未探到的参数/路径射点。**未读到任何题解**;不硬造。
- 待试面:输出重定向式探测确定 app 是否真在过滤;找 app 真实 docroot 读源;
  试 POST/其他方法或 `;` 参数分隔是否绕过边缘归一化。

## 复现命令

```
lab_launch launch C29F5F30A0C79A94F6A847A63911BFBB403F8D60977B3E3B09110D166711A345 --jar <jar>
lab_http get "<inst>/post?postId=1" --jar <jar>            # 看 Back to Blog 的 JS URL
raw_http "https://<inst>/post?postId=1" \
  --request-line "GET /post?postId=1&x=A!'\"();{}[]<>&%41+*/,:;=@~^|$ HTTP/1.1" \
  --header "Host: <inst>" --header "Cookie: session=<s>" --quiet --out /tmp/probe.html
```
