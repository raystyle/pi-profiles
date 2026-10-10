---
metadata:
  node_type: memory
name: "Lab race partial construction arm A"
description: "race partial-construction (arm A) 未解出: 应用面与 403/500 行为、已排除路径(空token齐发与播撒、登录面、旁路)、环境时序约束与可复用驱动件"
last_updated: 2026-10-10T10:31:41+08:00
created: 2026-10-10T10:31:41+08:00
---

## 2026-10-10 lab-race-conditions-partial-construction (arm A) - 未解出

靶场: PHP 应用 (平台每请求约 0.25-1s, Connection: close)。目标: 绕过邮件验证拿到 ginandjuice 账号并删 carlos。终态: 未解出, carlos 未删, banner 未翻。

### 已确证的应用面
- 注册: POST /register (username/email/password/csrf, csrf 会话绑定)。username 唯一(重名报 "An account already exists with that username"), email 不唯一(同邮箱重复注册返回成功页)。
- email 校验严格: 必须合法且域为 ginandjuice.shop; 双 at、子域、exploit 域全被 "Invalid email address" 拒。
- 注册后账号不可用: 正确口令登录仍 "Invalid username or password"; 无 wiener:peter 种子。
- POST /confirm?token=X: 400 "Incorrect token: X" (未命中); token 为空串或空数组 -> 403 "Forbidden"; token 含 NUL (%00) -> 500; token[]=x -> 400 "Incorrect token: Array" (PHP 侧比较迹象, 非 PDO 绑定)。
- GET /confirm?token=X 只渲染通用 "click the button" 页, 不校验; confirm 只读 query 的 token (空 query 加 body token 仍 403)。
- /admin 与 /admin/delete 无会话时 401 "Admin interface only available if logged in as a GinAndJuice user" (门禁正常)。

### 已排除的路径(逐条实测)
- confirm 空/0/null/NULL/false/undefined/Array/32hex 及请求派生值(用户名/邮箱/csrf/口令/session id): 全部 400/403, 含 100 流单包齐发、120s 连续播撒 (570 次空 token 全 403)、同会话空 token。
- 登录面窗口: 注册期间并发登录 (单会话串行 13 次 / 驱动 60 次 / 六账号并发验证) 全失败; 同会话读 /admin 与 /my-account 全 401/302。
- 非竞态旁路全负: 注册表单 mass assignment (confirmed/verified/token) 无效; token 参数无 SQLi (LIKE/引号/反斜杠/UNION 均干净, 但 NUL 500 说明 token 进过拒 NUL 的上下文); 无 token-as-path 行为; 用 carlos 邮箱注册不覆盖种子行; 邮箱域解析绕过被挡; 实验室邮箱只显示 exploit 域收件 (空)。

### 环境事实
- 每请求约 0.25-1s (Connection: close => 每请求新 TLS), HTTP/1.1 播撒被压到约 5 次/s; h2 单连接多流廉价, 但同一包内 register 在首 => 读侧多半排其后, 窗口不可观测; 需 h2 定速放流 (pacing) 才能对齐 late 窗口。
- h2 可用 (h2_probe 200, ALPN); h2_burst 100 流约 6s 出齐响应。
- 新件: .pi-rs/rust-scripts/pcreg_race.rs (注册 + 确认播撒 + 登录面并发 + 自动复验 + /admin 删除的驱动, selftest 通过), 本题未解出, 可复用于半构造注册族。

