---
metadata:
  node_type: memory
name: "PortSwigger lab launch and transport quirks"
description: "更正:unkeyed-param 件在 exploiting-implementation-flaws;路径错时用 sitemap.xml 枚举;widgets 仅 POST 形且不含题名"
last_updated: 2026-10-09T12:25:15+08:00
created: 2026-10-09T01:29:54+08:00
---

- range_launch 模板中毒坑(2026-10-09 实测 lab-csp-bypass,1.3.1 行为;1.3.2 已修:实例 host 判据收紧为纯小写 hex,题面 YOUR-LAB-ID 模板不再误捞):`launch-url <path>` 会把题面正文里形如 `https://YOUR-LAB-ID.web-security-academy.net/?...` 的模板当 instance_url 返回(instance_from_body:true),`launch <lab_id>` 则落到 /web-security/。可靠回退:http_session get `https://portswigger.net/academy/labs/launch/<lab_id>?referrer=<urlencoded path>` --follow,第一跳 302 的 Location 就是实例根(第二跳带上实例 session cookie)。


## 2026-10-09

- `/api/widgets` bare GET returns 404 (only specific lab widget paths resolve); widget-source fallback must use the lab page's own widget URL, not the bare API root.
- Canonical path given for `lab-web-cache-poisoning-with-an-unkeyed-param` under `/web-security/web-cache-poisoning/exploiting-design-flaws/` 404s — the real slug for this lab family differs (candidates: `lab-web-cache-poisoning-unkeyed-param`, or it lives under a different design-flaw sub-slug); resolve by title from the cache-poisoning index page rather than guessing.

## 2026-10-09

- 更正(2026-10-09):cache-poisoning 的「unkeyed param」件真实路径为 /web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-unkeyed-param(题名 Web cache poisoning via an unkeyed query parameter);此前记的「design-flaws 下 slug 不同」系段位错——同一 slug 尾巴落在 exploiting-implementation-flaws。
- 路径错时的权威枚举面:https://portswigger.net/sitemap.xml(一次拉全站 lab URL,cache-poisoning 13 条一网打尽),胜过逐 slug 猜。
- /api/widgets 只有 POST 形可用:体 [{widgetId:"academy-labstatus"|"academy-launchlab",additionalData:{"widget-lab-id":"<64hex>"}}] + 头 Widget-Source:<lab-page-path>;裸 GET 404。academy-labstatus 只回难度与 solved 态、academy-launchlab 只回 /academy/labs/launch/<id> 链接,均不含题名,故不能按题名反查——题名→路径只能靠 sitemap/页面。
