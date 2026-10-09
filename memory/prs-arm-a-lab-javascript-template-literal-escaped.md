---
metadata:
  node_type: memory
name: "PRS arm A lab-javascript-template-literal-escaped"
description: "arm A:lab-javascript-template-literal-escaped 冷实例一次通过 - ${alert(1)} 插值面绕转义,page_alert fired + banner solved"
last_updated: 2026-10-09T07:04:12+08:00
created: 2026-10-09T07:04:12+08:00
---

## 2026-10-09 arm A solve

- 题:lab-javascript-template-literal-angle-brackets-single-double-quotes-backslash-backticks-escaped(Reflected XSS into a template literal with angle brackets, single, double quotes, backslash and backticks Unicode-escaped)
- 实例:range_launch launch-url 直接吃 canonical academy 路径,reused:false,一次性拿到 instance_url;exploit_server 为 null(无利用服务器)
- 反射点(`?search=`),实测原文:`var message = \`0 search results for 'zzztestzzz'\`;` —— 值落在模板字面量内
- 解法:模板字面量的转义只覆盖引号/尖括号/反斜杠/反引号,`${}` 插值面未动 ⇒ 载荷 `%24%7Balert(1)%7D`(`\${alert(1)}`)直接被求值
- 判定:page_alert fired=true(alerts:["alert:1"],ready_state=complete);banner_verdict solved:true,congrats_line "<h4>Congratulations, you solved the lab!</h4>"
- 纪律:未读题解/既有实录;HTTP 全走件(range_launch/http_session/http_dump/page_alert/banner_verdict)

