---
title: lab-javascript-url-some-characters-blocked
---

# lab-javascript-url-some-characters-blocked

> evidences: [[xss-context-family]]

- 题面:Reflected XSS in a JavaScript URL with some characters blocked(/web-security/cross-site-scripting/contexts/lab-javascript-url-some-characters-blocked)
- 实例(批45R):https://0a2700ef04f2b73b8041d01f0011008e.web-security-academy.net
- 判定目标:反射 XSS 调 `alert`,消息含 `1337`;状态:**stuck**(编码器语义已量化到字符级;JS 串在可用字母表下不可破)

## 射点(唯一反射点)

```html
<a href="javascript:fetch('/analytics', {method:'post',body:'/post%3fpostId%3d1%26x%3d<PAYLOAD>'}).
   finally(_ => window.location = '/')">Back to Blog</a>
```

- 整个「请求 URI」被重序列化后落进 **JS 单引号字符串**;标记物在整份响应里只出现 1 次(text_grep 佐证)⇒ 没有第二射点。

## 编码器字母表(逐字符实测)

| 处理 | 字符 |
| --- | --- |
| 原样保留 | 字母数字 与 `" { } \| ^ $ , * / ~ _ - . !`(空格→`+`) |
| 百分号编码 | `'`→%27 `;`→%3b `:`→%3a `=`→%3d `@`→%40 `<`→%3c `>`→%3e `&`→%26 `+`→%2b `?`→%3f |
| 删除 | `( ) [ ]` 反引号 `\` 与**所有 `%`** |

- 管线 = **先百分号解码一次,再编码/删除**;`x=A%25%32%38%25%32%39B` → `A2829B` ⇒ 双重编码拿不到字面 `%28`(`%` 必被删)。
- 现代 Chrome **不对 `javascript:` URL 做百分号解码**(复测:`a.href="javascript:alert%281337%29"` + click → 无 alert)⇒ `%27` 当不了 `'`。

## 未决面(卡点收窄为「一个字符」)

- `"` 可破 HTML 属性,但 `=` 被编码 ⇒ 只能注入属性名、给不出事件处理器值;`&` 被编码 ⇒ 用不了 HTML 实体 `&#39;`;`'`/`\`/`%` 全不可得 ⇒ **无法在 href 里重开或闭合那个单引号**。缺的杠杆:任何能把 `'`(或可被浏览器解码的 `%27`)送进 href 的通道。
- 待验:真用户手势点击下 Chrome 是否对 `javascript:` 做百分号解码(批45R 只验了合成 `a.click()`;若真则本卡的唯一障碍只剩「把 %28/%27 送进 href」)。

## 证据摘录

```
href(探针 [ ] ( ) 反引号 \ % 等): javascript:fetch('/analytics', {method:'post',body:'/post%3fpostId%3d1%26x%3d%27"%3b%3a%3d%40%3c%3e%26%2b{}|^$/+2829'})…
x=A%25%32%38%25%32%39B → …%3dx%3dA2829B'      # 双重编码 → 剩余 hex 数字
browser_suite eval: javascript:alert%281337%29 点击 → hook []
```

## 复现命令

```
range_launch launch C29F5F30…6711A345 --jar ~/.pi-rs/agent/chrome-jar.json
http_session get "https://<inst>/post?postId=1&x=<探针>" --jar <jar> --out /tmp/l4.html
text_grep 'javascript:fetch' /tmp/l4.html
```

## 关系

- 族:[[xss-context-family]] 的「`javascript:` URL 射点」行(该行结论按本档改写:编码集含 `&`/`?`,且 `%` 被删)。
