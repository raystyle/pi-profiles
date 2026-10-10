---
metadata:
  node_type: memory
name: "Lab: logic-flaws auth bypass via flawed state machine"
description: "State-machine login bypass: skip POST /login's /role-selector state → /admin defaults privileged; carlos deleted, banner solved."
last_updated: 2026-10-10T02:51:13+08:00
created: 2026-10-10T02:51:13+08:00
---

## 2026-xx (arm A, portswigger lab-logic-flaws-authentication-bypass-via-flawed-state-machine)

- Path: `/web-security/logic-flaws/examples/lab-logic-flaws-authentication-bypass-via-flawed-state-machine`; range_launch launch-url 直接起实例(reused:false),jar /tmp/cj1.json。
- 状态机取形:GET /login 取 csrf → POST /login(csrf+wiener:peter) 原始响应 302 `Location: /role-selector`,同时 `Set-Cookie: session=<新值>` —— 凭据通过后进入「选角色」态。
- 缺口:不访问 /role-selector,直接带该新会话 GET /admin → 200,页面标题栏显示 `My account → /my-account?id=administrator`,用户表含 wiener 与 carlos(未初始化角色被当成 administrator)。
- 收束:GET `/admin/delete?username=carlos` → 302 `/admin`;banner_verdict 对实例根 → `solved:true`,`<h4>Congratulations, you solved the lab!</h4>`。
- 泛化:任何「凭据校验后还有后续态(选角色/确认页/第二步)」的登录流,先记录 POST 的 302 target 并刻意跳过它,用新会话直取终态资源;状态变量未赋值时的默认值往往就是最特权值。

