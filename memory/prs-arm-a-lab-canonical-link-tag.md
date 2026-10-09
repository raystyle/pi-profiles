---
metadata:
  node_type: memory
name: "PRS arm A lab-canonical-link-tag"
description: "arm A:lab-canonical-link-tag 冷实例一次通过(%27 破单引号属性注 accesskey+onclick,page_alert 触发,banner solved;无 exploit server)"
last_updated: 2026-10-09T00:24:33+08:00
created: 2026-10-09T00:24:33+08:00
---

arm A 基线:lab-canonical-link-tag 冷实例(reused:false)一次通过 - 首页 canonical 属性用单引号,`/?%27accesskey=%27x%27onclick=%27alert(1)` 的 `%27` 被服务端解码成裸 `'` 破属性,注入 accesskey+onclick;page_alert 在该 URL 上 click() 等价触发得到 alerts:["alert:1"],随后 banner_verdict solved:true(congrats)。题面只说模拟用户按 ALT+SHIFT+X,实例与 academy 原页均无 exploit server 入口,无需交付面。
