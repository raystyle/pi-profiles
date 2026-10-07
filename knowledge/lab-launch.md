---
title: 靶场实例发射:widget 渲染与 OIDC 回放
---

# 靶场实例发射:widget 渲染与 OIDC 回放

PortSwigger Academy 的实例发射全程纯 HTTP 可达,不需要浏览器引擎,也不走
browse/CDP 面。属工具面,入口是账户会话。

## 发射链路

1. 发射按钮由客户端 widget 渲染:`POST /api/widgets`,体为页面所有
   `[widget-id]` 的 `{widgetId, additionalData}`,头带
   `Widget-Source: <题页路径>`;`academy-launchlab` 的返回 HTML 内嵌
   `/academy/labs/launch/<lab-id 小写>?referrer=...`。
2. GET 该发射路径。匿名 -> 302 `/users?returnurl=...` ->
   `login.portswigger.net/authorize`(Auth0)。
3. 持有效 `login.portswigger.net` `auth0` 会话 cookie 时,authorize 返回
   200 自动提交表单(`response_mode=form_post`,隐藏域 code/state);
   POST 该表到 `/signin-oidc` -> 302 `/auth0/complete?returnUrl=...`,并下发
   `.AspNetCore.CookiesC1/C2`(应用登录态)。
4. GET `/auth0/complete` -> 302 到 `https://<id>.web-security-academy.net/`。

## 会话来源与工具面

- 会话 cookie 由离线解密 Chrome v10 库取得(16 字节 `peanuts` 派生键,
  AES-128-CBC,前缀 16 字节先剥后去 PKCS7)。
- 裸 HTTP 交互用 get/post/submitform 型的 cookie-jar 客户端;发射与 OIDC
  回放可整链封装,收尾用实例横幅判读。

## 陷阱

- 发射受账户门禁:匿名必然 302 到 Auth0;`auth0` 会话有效期约 3 天。
- `/auth0/complete` 偶发 CloudFront 403(Bad request),是传输层抖动,
  单发 GET 重放即过,不是鉴权失败。
- 同一账户重复发射通常返回既有实例而非新建。

## 利用服务器与邮件客户端(配套面)

- 利用服务器:`POST /` 表单 `formAction=STORE|DELIVER_TO_VICTIM`(`responseFile/responseHead/responseBody`);
  结果读 `GET /log`(交付是否被 bot 访问)。
- 邮件客户端:`https://exploit-<id>.exploit-server.net/email`(读发给受害者/攻击者的确认信,抽 token)。
- widget-lab-id -> 题名/URL:POST `portswigger.net/api/widgets` `academy-labstatus`。
- 活跃 PSW 会话是短命件(如 `/tmp/*-jar.json`),跨批不可依赖;会话底座见 [[prototype-pollution-family]] 的发射链段。
