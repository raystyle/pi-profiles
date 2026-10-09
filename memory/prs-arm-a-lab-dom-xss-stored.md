---
metadata:
  node_type: memory
name: "PRS arm A lab-dom-xss-stored"
description: "arm A 基线:lab-dom-xss-stored 冷实例一次通过 - 评论体 `<><img src=1 onerror=alert(1)>` 吃掉首个 `<`/`>` 替换即弹 alert,banner solved"
last_updated: 2026-10-09T07:08:32+08:00
created: 2026-10-09T07:08:32+08:00
---

arm A 基线,lab-dom-xss-stored(Stored DOM XSS)。

- 实例:range_launch launch-url /web-security/cross-site-scripting/dom-based/lab-dom-xss-stored --jar /tmp/cj1.json → https://0a23009d0378f8a9809e03740026004e.web-security-academy.net/,reused:false。首跑漏 --jar 落到 login.portswigger.net(信封 instance_url:null,final_url 是 login 域)——回退信号。
- 病灶:渲染件 /resources/js/loadCommentsWithVulnerableEscapeHtml.js,escapeHTML = html.replace('<','&lt;').replace('>','&gt;'),字符串形各只换第一处;comment.body 进 innerHTML。
- 命中:POST /post/comment(csrf+postId=1+comment=`<><img src=1 onerror=alert(1)>`+name+email) 302 → /post/comment/confirmation?postId=1;GET /post/comment?postId=1 JSON 里 body 为原文;page_alert /post?postId=1 --settle-ms 3000 → fired:true alerts:["alert:1"];banner_verdict solved:true。
- 件组合:range_launch → http_dump(读渲染脚本) → http_session(提交+读回 JSON) → page_alert(触发判据) → banner_verdict(收口)。无需新件。
- 无 exploit server(victim 不需投递),题库存储侧不编码、编码只在渲染层。

