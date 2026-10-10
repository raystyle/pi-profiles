---
metadata:
  node_type: memory
name: "PRS arm B lab-web-cache-poisoning-combining-vulnerabilities"
description: "arm B 开卷: combining-vulnerabilities 一次解出 - 两段序(片段形投毒翻 lang cookie 再投 exploit 形) + XFH 经 # 令 fetch 落同源 /setlang"
last_updated: 2026-10-09T13:11:54+08:00
created: 2026-10-09T13:11:54+08:00
---

# PRS arm B: lab-web-cache-poisoning-combining-vulnerabilities

日期: 2026-10-09 (arm B, 开卷带注: 先读 cache-poisoning-family / cache-and-smuggling-live-mechanisms / internal-and-fragment-cache-poisoning-method 与 tactical-patterns P6/P7/P14)

题面: 用户每约 60 秒访问首页一次、语言为英语;须经缓存投毒在受害者浏览器执行 `alert(document.cookie)`。

实例与件: 目标实例 + 独立 exploit server;全部 HTTP 走件(range_launch launch-url / http_dump / cache_probe / http_session / poison_loop / browser_suite / nap)。

链路(实证 solved,两段序):
1. 首页缓存 `max-age=30` + `X-Cache`;缓存键 = 完整 URL(含 query),**X-Forwarded-Host 不入键**。XFH 被回显进内联 `data={"host":...}`(JSON 转义,`</script>` 也转义为 `<\/script>` → 无 JS 注入面),该值拼进 `initTranslations('//' + data.host + '/resources/json/translations.json')`。
2. DOM sink 在 translations.js:`el.innerHTML = dict[k]`,但仅当 `lang` cookie 存在且 `lang.toLowerCase() !== 'en'` 时才调用 translate。载荷必须是**裸** `<img src=1 onerror=alert(document.cookie)>`:写成 `&lt;` 实体会被 innerHTML 赋值当作文本(不生成元素),实测 onerr=0。
3. lang 只能由 `/setlang/<lang>`(302 + `Set-Cookie: lang=...`)设置;而缓存命中会剥掉 Set-Cookie(命中行 set_cookie 恒空),故缓存体永远无法给受害者种 cookie。
   关键突破: XFH 值允许 `/` 与 `#`:`X-Forwarded-Host: <TARGET>/setlang/en-gb#` → 拼出的 fetch URL 去掉 fragment 后 = **同源** `https://<TARGET>/setlang/en-gb` → 同源 fetch 落地 Set-Cookie → 受害者浏览器 `lang=en-gb`。自证: browser_suite goto 该投毒页后 `document.cookie` = `lang=en-gb`。
4. 两段序: 先按 1s 循环把 `/` 投成 fragment 形(翻 lang),再按 1s 循环投成 exploit 形(跨源 JSON + `Access-Control-Allow-Origin: *`),受害者下一次访问即翻牌。
5. exploit server 路径敏感(只有 responseFile 路径可读,其它 404),用 `--form responseFile/responseHead/responseBody` STORE;响应用 `Content-Type: application/json` + ACAO:*。

坑与读法边界:
- 缓存命中不续 TTL(旧变体先进时,同变体循环只能等 ~30s TTL 过期才换得掉);换变体前须让旧条目过期(约 30s 不动 `/`),但受害者可能抢先把 miss 写成干净页 → 换变体要用 1s 级循环抢 miss 窗口。
- 别用裸根路径校验 solved(会把干净页写回挤掉投毒),校验读带随机参数的 URL。
- 受害者只访问目标首页、**不来 exploit server**(deliver 后无 bot 日志)→ 不能靠 exploit 页重定向翻转 lang;但无 bot 日志也 ≠ 失败,判据是 exploit server 日志里出现受害 UA 抓 `/resources/json/translations.json`,终判恒 banner `is-solved`。
- 本批未新建件;现有件(poison_loop / cache_probe / http_session / browser_suite)足够。未 git 提交。

