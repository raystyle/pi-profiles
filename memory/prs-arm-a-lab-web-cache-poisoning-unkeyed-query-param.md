---
metadata:
  node_type: memory
name: "PRS arm A lab-web-cache-poisoning-unkeyed-query-param"
description: "arm A 基线:lab-web-cache-poisoning-unkeyed-query-param 冷实例一次通过 - utm_content 出缓存键且查询串解码后原样反射进 canonical 单引号属性,poison_loop 2s×35 后 banner solved"
last_updated: 2026-10-09T12:11:04+08:00
created: 2026-10-09T12:11:04+08:00
---

## 2026-10-09 arm A 基线:lab-web-cache-poisoning-unkeyed-query-param(一次通过)

- lab 路径:canonical 是 `exploiting-implementation-flaws/lab-web-cache-poisoning-unkeyed-param`(题面写 design-flaws + `...-unkeyed-query-param` 是 404 错 slug)。回退取号:抓 `/web-security/web-cache-poisoning/exploiting-implementation-flaws` 主题页,按 h3「Unkeyed query parameters」定位 widget-lab-id `F14385AB...A863`,range_launch launch 直接起实例(冷实例 reused:false)。
- 缓存面盘(cache_probe,X-Cache/X-Cache-Key):
  - `/` → key `/$$`;`/?utm_content=X` → key `/$$`(utm_content 被逐出缓存键);`/?foo=Y` → key `/?foo=Y$$`。
  - `Cache-Control: max-age=35`,响应永远 200。
  - `utm_content` 的值还会被写进 Set-Cookie(`utm_content=<v>; Secure; HttpOnly`)。
- 注入面:首页 `<head>` 的 `<link rel="canonical" href='//HOST/?<QS>'/>`,QS 是**整个查询串按百分号解码后原样写入**(实测 `%3C`→`<`、`%27`→`'`、`%22`→`"` 均落地),破单引号即出属性。
- 载荷:`?utm_content=x%27%3E%3Cscript%3Ealert(1)%3C%2Fscript%3E` → 服务端渲染成 `href='//HOST/?x'><script>alert(1)</script>'/>`。
  - 百分号编码是必需技巧:缓存键用**编码态**,payload 编码后更不容易污染键(且 utm_content 本就出键)。
- 落毒难点:`/$$` 条目被模拟受害者周期性访问刷新(等 40s 后 age 只有 13s,说明受害者在刷),单发几乎必 hit。解法=`poison_loop <url> --interval-secs 2 --count 35 --jar`(2s 一轮连刷 107s),任一轮撞上过期窗即写毒,受害者下一次访问命中翻牌。
- 证据:poison_loop 35 轮全 200 → banner_verdict `solved:true` / `Congratulations, you solved the lab!`。
- 坑:http_session 信封会剥 `<script>` 块(空反射槽≠未反射),判反射/判毒要 http_dump(带 X-Cache/Age)或 raw 读取;本次 poison 请求曾 hit(age 13)导致误判,靠 X-Cache 头纠正。

