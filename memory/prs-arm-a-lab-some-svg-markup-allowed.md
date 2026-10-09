---
metadata:
  node_type: memory
name: "PRS arm A lab-some-svg-markup-allowed"
description: "arm A baseline: lab-some-svg-markup-allowed 冷实例一次通过 - svg animateTransform onbegin 触发 alert,banner solved"
last_updated: 2026-10-09T00:41:24+08:00
created: 2026-10-09T00:41:24+08:00
---

## 2026-10-09 arm A 基线解

题:lab-some-svg-markup-allowed(canonical 路径 /web-security/cross-site-scripting/contexts/lab-some-svg-markup-allowed)。冷实例(reused:false,首判 banner 未解)。

链:
1. page_read 直取 widget-lab-id(64 hex)= 20812D51…72FFB。
2. range_launch 起实例。range_launch 自身回 transport error(timed out reading response),但已吐出实例 URL;随后 banner_verdict 200 证明实例其实已起 —— range_launch 的 post-launch 探测是 rs HTTP 栈 TLS 抖动,不是发射失败。
3. 反射点:GET /?search=… 落 `<h1>0 search results for '<payload>'</h1>`,HTML 文本上下文、单引号只是页面文案不构成属性边界。
4. 过滤面:黑名单式挡常见标签/事件;`<svg><animatetransform onbegin=alert(1)></animatetransform></svg>` 整体原样落地(body 回显无损)。
5. page_alert 携 payload URL → fired=true, dialog `alert:1`(animateTransform 无 begin 属性时 begin 默认 0s,SVG 时间线启动即触发 onbegin)。
6. banner_verdict → solved=true + congrats 行。

结论:一次通过(冷实例),无需新件。可复用要点 = SVG 标签名大小写不敏感(`animatetransform`)与 onbegin 事件不在黑名单;命中前先用 http_session 验反射无损再上 page_alert。

