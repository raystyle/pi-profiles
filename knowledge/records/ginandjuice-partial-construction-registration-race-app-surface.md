---
title: "GinAndJuice partial-construction registration race app surface"
---

# GinAndJuice partial-construction registration race app surface

PHP 注册/邮件验证应用(半构造注册竞态族, academy lab-race-conditions-partial-construction 实例). 观察到的稳定行为与不变量:

## 端点面
- `POST /register` (username/email/password/csrf, csrf 会话绑定): username 唯一性检查(重名报 "An account already exists with that username"); email 不唯一(同邮箱二次注册返回成功页); 成功页文案 "Please check your emails for your account registration link"。
- email 必须 `filter_var` 合法且域恰为 ginandjuice.shop; 双 at、`domain.ginandjuice.shop` 子域形、exploit 域一律 "Invalid email address"。
- `POST /login` (csrf 会话绑定): 未验证账号用正确口令也返回 "Invalid username or password"(状态被隐藏); 无种子 wiener 账号。
- `POST /confirm?token=X`: 精确匹配; 未命中 400 `"Incorrect token: X"`(回显输入); token 为空串或空数组时 403 `"Forbidden"`; token 含 NUL(%00)时 500; token[]=x 得 400 "Incorrect token: Array"; 只读 query 的 token(空 query + body token 仍 403, body 不覆盖)。
- `GET /confirm?token=X`: 只渲染通用 "click the button" 页, 不校验 token。
- `/admin`, `/admin/delete`: 无会话或非 GinAndJuice 用户 -> 401 "Admin interface only available if logged in as a GinAndJuice user"。

## 不变式与推论
- 空/nil token 走独立分支(403), 无法用它命中"未写入 token 的半构造行"。
- token 到达过拒绝 NUL 的上下文(500), 但语法面干净: 无 LIKE 通配、无引号/反斜杠/UNION 注入, 无 token-as-path 行为。
- 注册时刻的中间态对 confirm 与 login 两条读面都不可观测(注册窗口内并发读未取到任何非基线响应)。
- 已验证注册后的账号处于不可登录的未验证态, 且该状态按用户行而非按邮箱判定(用种子用户邮箱注册同样不可登录)。

## 相关
- 驱动件: `.pi-rs/rust-scripts/pcreg_race.rs`(注册 + token 电池播撒 + 登录面并发 + 自动复验 + /admin 删除)。
- 可用原语: h2 单包齐发(`h2_burst`), HTTP/1.1 屏障齐发(`race_send`), 读侧播撒(`race_spread`)。
- 环境约束: 目标 Connection: close(每请求新 TLS, HTTP/1.1 播撒约 5 次/s), h2 多流单连接是唯一廉价高并发读通道。
