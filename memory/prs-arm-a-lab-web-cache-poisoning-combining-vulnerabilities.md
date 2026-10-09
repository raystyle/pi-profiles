---
metadata:
  node_type: memory
name: "PRS arm A lab-web-cache-poisoning-combining-vulnerabilities"
description: "arm A:lab-web-cache-poisoning-combining-vulnerabilities 一次解出 - XFH 投毒 data.host + /setlang 翻转受害者 lang cookie + exploit server 加 ACAO:* 供跨源 translations.json,XSS 经 translate() innerHTML"
last_updated: 2026-10-09T11:35:47+08:00
created: 2026-10-09T11:35:47+08:00
---

## 终态

solved（banner `is-solved` + "Congratulations, you solved the lab!"）。实例冷启（reused:false），命中链路 = 缓存投毒 `data.host` → 受害者浏览器从 exploit server 取 malicious translations.json → 既有 DOM XSS sink。

## 链路（三段，缺一不通）

1. 首页 / 被缓存（`max-age=30`，键 = path+query），页内 `data = {"host":...,"path":"/"}` 的 host 取自 **`X-Forwarded-Host`**（unkeyed，逐字进 JSON，`"`/`\` 被 JSON 转义、`/`→`\/`，不能借引号破 script；Host 头换成外域直接被前端 403）。页尾 `initTranslations('//' + data.host + '/resources/json/translations.json')` → 投毒即让受害者去我们的 server 取 JSON。
2. **语言门**：translations.js 只在 `lang in j && lang.toLowerCase() !== 'en'` 时才跑 `translate()`，而受害者语言是 en ⇒ 必须先把受害者的 lang cookie 换掉。做法：把 `data.host` 投成 `<labhost>/setlang/es?`，受害者的 fetch URL 变成同源 `/setlang/es?/resources/json/translations.json` → 302 `Set-Cookie: lang=es` 落在受害者 jar（XHR 同源，Set-Cookie 生效），下一轮访问才投 exploit host。
3. **CORS 门（自测才发现）**：该 fetch 跨源，exploit server 的响应必须带 `Access-Control-Allow-Origin: *`，否则 JSON 根本 parse 不到、translate 不执行。另外 exploit server 只在 `responseFile` 的精确路径上回存储体（默认 `/exploit` → 取 `/resources/json/translations.json` 得 404），且 STORE 必须带 `formAction=STORE`（缺 → 400 Missing parameter responseFile/formAction）。

## 判据与证据

- 投毒是否生效用 `http_dump /` 看 `data.host`（命中即 victim 也会拿到）；exploit 访问日志出现 `Mozilla/5.0 (Victim)` 的 `GET /resources/json/translations.json 200` 后横幅翻 is-solved。
- 有效载荷 = 每个语言码的 `translations` 里给 `"View details"`（首页锚 innerHTML 恰为该串）与 `"Return to list"`/`"Description:"`（商品页）赋 `<img src=1 onerror=alert(document.cookie)>`。

## 坑

- 首次 header 扫描假阴性：`header_fuzz` 打的是已热 `/` 条目，25 个变体全是 hit ⇒ 探反射必须用唯一 `?cb=N` cache-buster 逼后端。
- 缓存 TTL 30s + 受害者约 1 分钟一访：poison 请求每 ~5s 一轮才压得住；若受害者自己的 MISS 先写了一条干净条目，我们下一轮就是 hit（30s 内投不进去）。
- 自测手法（省事且不扰现场）：CDP 里 `Network.setCookie` 设 lang + `window.alert=fn(a){window.__xss=String(a)}` 后手动调页面自带的 `initTranslations('<exploit>/resources/json/translations.json')`，再读 `window.__xss` —— 非阻塞地证 sink 通不通。

