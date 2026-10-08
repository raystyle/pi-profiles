---
title: lab-race-conditions-partial-construction
---

# lab-race-conditions-partial-construction

> evidences: [[race-conditions-family]]

- 题面:Partial construction race;绕邮箱验证建号 → 登录 → 删 carlos。
- 判定:**stuck**(空 token 被 app 层 403 挡死;真单包 login 与 confirm 两个 oracle 均干净否证)

## 端点事实

1. 注册表单由 `/resources/static/users.js` 的 `createRegistrationForm()` 生成(csrf/username/email/password);`confirmEmail()` 把 URL query **原样**拼成 `POST /confirm?<query>`(form 无字段)。注册成功页 2636B "Please check your emails for your account registration link";重复用户名页 3142B;email 白名单只收 `@ginandjuice.shop`(exploit 域 3119B 拒)。
2. csrf 会话级;`POST /login` **强制 csrf**(缺 → 400 "Missing parameter 'csrf'";会话无 csrf → 400 "Invalid CSRF token (session does not contain a CSRF token)")⇒ 每条腿需自己的 session+csrf。
3. `/confirm` **只认 query**:`?token=AAA` + body `token=BBB` → "Incorrect token: AAA";`POST /confirm`(无 query)+ body token → "Missing parameter: token";`?token=`(空)→ 403。
4. 空值语义:login 空 password → 400 "Missing parameter";confirm 空 token → 403 "Forbidden";`token[]=`/`token[]` → 400 "Incorrect token: Array"(数组当 `''` 铺开);`token=0`/`1`/`null`/`undefined` → 400 "Incorrect token: <原样>"。
5. login 失败页 3801B "Invalid username or password"(不区分不存在/未确认)。

## 真单包否证

- `h2_burst` 用 **6 条独立 session+csrf** 的 1 register + 5-8 login(`single_packet_likely=true`,burst_bytes 1319)⇒ register 200/2636,**login 全 200/3801,零 302**。
- 真单包 1 register + 6× `POST /confirm?token[]=`(burst_bytes 823)⇒ confirm **全 400 "Incorrect token: Array"**。
- 真单包 1 register + `token=0`/`1`/`null`/`undefined` ⇒ 全 400。
- ⇒ 半构造行的 token 既不是 `''`(否则 `token[]=` 命中)也不是 `0/1`;login 在窗口内不被放行(confirmed 自 INSERT 起即为 0)。**时序维度彻底用尽**(跨批累计确认腿数百次、登录腿真单包)。

## 判读与下一步

- 未决面:被后置写入的列是哪一列、其窗口内可匹配形态;需要一个**非变异、能区分半构造行**的读原语(该 lab 的 HTTP 面目前没有)。
- 复攻建议:换维度(能回显用户表的端点/错误码差分),不再加采样、不再换并发模型。

## 关系

- 族:[[race-conditions-family]];方法见 [[http-2-single-packet-race-burst-method]]、[[visibility-probe-must-be-non-mutating]]。
