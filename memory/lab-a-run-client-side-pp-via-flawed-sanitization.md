---
metadata:
  node_type: memory
name: "Lab A-run: client-side PP via flawed sanitization"
description: "A 臂实录: PP via flawed sanitization - 单遍 replaceAll 绕过 __pro__proto__to__, gadget config.transport_url -> data:,alert(1), solved"
last_updated: 2026-10-09T20:25:26+08:00
created: 2026-10-09T20:25:26+08:00
---

## 2026-xx-xx A 臂实录 (lab-prototype-pollution-client-side-...-via-flawed-sanitization)

实例: https://0aa400e403341b528151343300c40053.web-security-academy.net/ (range_launch launch-url, reused:false)

题面读法: page_read canonical 路径 -> lab_id E62603F6... -> range_launch launch-url 起实例。

源码面 (http_dump 两个资产):
- /resources/js/deparamSanitised.js = source。deparam 解析 location.search,写 obj[sanitizeKey(key)]。
  sanitizeKey 用 replaceAll 单遍删 ['constructor','__proto__','prototype'] —— 非递归单遍,可构造自重叠串绕过。
- /resources/js/searchLoggerFiltered.js = gadget。config = {params: deparam(...)};
  if (config.transport_url) { script.src = config.transport_url; body.appendChild(script); }

机制 (两句):
- 绕过: `__pro__proto__to__` 中索引 5..13 命中 `__proto__`,单遍删除后剩 `__proto__`;
  第二次 prototype 替换不再命中 -> 落回原键。
- 投毒: 循环 i=0 读 obj["__proto__"] 拿到 Object.prototype(真值),cur 指向它;
  i=1 写 transport_url -> 落在 Object.prototype 上。config.transport_url 走原型链命中。

载荷 (page_alert, 一次命中 fired=true alerts=["alert:1"]):
?__pro__proto__to__%5Btransport_url%5D=data:,alert(1)

判定: banner_verdict solved=true, congrats_line="Congratulations, you solved the lab!"。

件耗: page_read 1, range_launch 1, http_dump 3, search_content 2, read 2, page_alert 1, banner_verdict 1。
教训: 单遍 replaceAll 黑名单 = 自重叠串绕过;source 键路径的读取语义(.  vs [])决定污染是否落到原型上。

