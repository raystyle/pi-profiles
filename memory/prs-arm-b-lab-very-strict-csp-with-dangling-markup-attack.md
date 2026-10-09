---
metadata:
  node_type: memory
name: "PRS arm B lab-very-strict-csp-with-dangling-markup-attack"
description: "arm B 冷实例一次通过:表单劫持向量 + 关键坑(因果已修正):受害者爆发稀疏带 cooldown,链须单次访问内自闭合——浏览器交付是可用形非必要条件"
last_updated: 2026-10-09T03:42:32+08:00
created: 2026-10-09T03:42:32+08:00
---

- 2026-10-08 arm B,实例 https://0a7800fe0351a8c686a5808f00be002a.web-security-academy.net/ ,终判 banner "Congratulations, you solved the lab!"(solved:true)。
- 反射点:`/my-account?email=<v>` 原样落进 email input 的 `value="..."`(双引号与尖括号都不编码,CSP 页无输出过滤);页面 CSP = `default-src 'self';object-src 'none';style-src 'self';script-src 'self';img-src 'self';base-uri 'none'` —— **没有 form-action** ⇒ 表单可提交到任意外域,而 img/link/iframe 类外呼全被挡 ⇒ 本题没有免点击外传信道。
- 解法(表单劫持,一次交付即 solved):向量 `hacker@evil-user.net"><button class=button type=submit formnovalidate>Click me</button><input type="submit" value="Click me" formnovalidate>`。注入点本来就在 change-email 表单内,点任一提交控件即用受害者自己的 session+csrf POST `/my-account/change-email`,受害者邮箱改成 hacker@evil-user.net。
- 交付页:`<meta http-equiv="refresh" content="0;url=<上面那个 URL>"><a href="<同上>">Click me</a>`(自动跳转把受害者送进 lab 面,anchor 仅作后备)。
- **本批最大的坑(因果修正,kimi 审计 F1)**:~~只有浏览器驱动的 Deliver 才召来受害者~~——此因果被同日反证(128 B-1 裸 POST+--follow 即召来并 solved;本题自身 [19] 首次 http_session 交付后 19:21 亦有受害者爆发)。当前假说(待二证):受害者爆发稀疏带 cooldown,重复交付不再召来;载荷链须在单次访问内自闭合(auto-nav+一次点击)。浏览器驱动交付是可用形之一,不是必要条件。同一张 exploit server 表单,用 CDP(HeadlessChrome)点 `button[value=DELIVER_TO_VICTIM]` ⇒ 1 秒后 10.0.3.168 / UA `Mozilla/5.0 (Victim) Chrome/154` 就来取 `/exploit/`,随后立刻 solved;而用 `http_session` 裸 POST `/`(formAction=DELIVER_TO_VICTIM) 连做 6 次,受害者一次都没来。以后这类 lab 的交付走 browser_suite 真浏览器,别用裸 HTTP。
- 早前 19:17:45→19:21:50 曾有一次性 4 分钟访问潮(每 3.5s 取一次 `/exploit/`,共 65 次),期间注入向量是 `<button>` 表单劫持却始终 not solved ⇒ 该访问潮里没有落地点击(点击预算被我们页面的 anchor 吃掉,或页面被自动跳转带走);真正生效的点击只出现在「浏览器交付 + meta 跳转落到 lab 面」那次。
- 另证:CSRF 会话绑定 —— 新会话带旧会话的 token → 400 `Invalid CSRF token (session does not contain a CSRF token)`;所以只能借受害者浏览器同源提交,自己重放没用(也就解释了描述里的「偷 token」说法)。
- 本地验收手段(很有用):`browser_suite call Network.setCookie`(注入 session)→ `goto` 向量 URL → `eval` 点注入的 button ⇒ 页面跳到 `/my-account?id=wiener` 且邮箱已变,证明载荷本身没问题,问题全在平台侧触发/点击。
- 封死的旁路:注入的 `<meta http-equiv=refresh>` 在 lab 面(位于 form 内、body 里、受该 CSP)不执行 —— 实测 DOM 里有该 meta 且 content 里吞到了 csrf token,但浏览器没有导航 ⇒ 免点击 exfil 路线在这题不可用。
- 纪律:全程走件、未 git 提交;本轮未读本题任何既有 records/memory/practice 档案。

