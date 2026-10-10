---
metadata:
  node_type: memory
name: "PRS arm A lab-web-cache-poisoning-fat-get"
description: "arm A 基线:lab-web-cache-poisoning-fat-get 冷实例(reused:false)一次通过 - GET 带体 callback=alert(document.cookie) 投毒 /js/geolocate.js?callback=setCountryCookie,banner solved"
last_updated: 2026-10-09T13:16:35+08:00
created: 2026-10-09T13:16:35+08:00
---

## 2026-10-09 arm A baseline - lab-web-cache-poisoning-fat-get

路径:`/web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-fat-get`(canonical,直填即中)。range_launch launch-url 冷实例 reused:false,实例 0a7200a8035b1c9881e6357000ff0077,无 exploit server。

### 面盘
- 首页被缓存(X-Cache hit,max-age=35),内联 `<script src="/js/geolocate.js?callback=setCountryCookie">`。
- `/js/geolocate.js?callback=setCountryCookie` 基线 201B:`setCountryCookie({"country":"United Kingdom"});` —— callback 参数决定被调函数名,响应也入缓存(max-age=35)。
- 缓存键 = 方法 + URL,忽略请求体:带体的 GET 与不带体共享同一个键。

### 解法(件驱动)
1. `fatget_poison <script-url> --body 'callback=alert(document.cookie)' --header 'Content-Type: application/x-www-form-urlencoded' --interval-secs 10 --count 30`
   - 语义确认:首轮 x_cache=hit(旧净体 201B),次轮 x_cache=miss 且 body_len=207(+6 = `alert(document.cookie)` 22 字符替 `setCountryCookie` 16 字符)⇒ 源站确实取了体内 callback。
   - 以独立无体 GET 回读命中项,得 `alert(document.cookie)({"country":"United Kingdom"});`(先弹 alert,再调 undefined 抛错,不影响触发)。
2. 投毒必须在 cache miss 那一轮落盘,故用 10s 间隔(远小于 35s max-age)保持条目长期为毒:命中轮不刷新,过期间隙由下一轮 miss 补上。
3. 受害者约每分钟首访首页一次;首访后 `banner_verdict <base> --jar /tmp/cj1.json` → solved:true。

### 坑
- `raw_http ... --body 'callback=...'` 对本题返 400 `Missing param 'callback'`:body 未被源站解析(缺/畸 Content-Type 面),但足以证明「带体时查询串被丢弃」这一行为。同类题带体走 `fatget_poison`(ureq `send_string` 自动 Content-Length + 显式 Content-Type)最稳。
- 别用一次投毒就等:35s 过期后条目会退回净体,须保持轮询直到 banner 翻。
- 时序:受害者的无体请求若在毒条目过期瞬间到达,会自己把净体写回;短间隔重投可把这个窗口压到最小。

### 件
`fatget_poison` 1.0.0 足够,无需新件。

