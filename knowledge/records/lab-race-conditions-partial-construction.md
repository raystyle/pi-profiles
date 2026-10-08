---
title: lab-race-conditions-partial-construction
---

# lab-race-conditions-partial-construction

> evidences: [[race-conditions-family]]

- 题面:Partial construction race;绕邮箱验证建号 → 登录 → 删 carlos。
- 实例(批51):https://0a0a00750454f6208085219700d10070.web-security-academy.net
- 判定:**stuck**(确认腿 = `POST /confirm?token=…`;空 token 被 app 层 403 挡死 ⇒「半构造行 token 为空串」这条路在本实例不成立,读侧播撒也没找到窗口)

## 端点事实(批51 实测,合并批50)

1. **注册**:`GET /register` 表单由 `users.js` 的 `createRegistrationForm()` 生成;csrf 是**会话级且可重复用**;新 INSERT 页 2636B、重复用户名页 3142B(唯一判据二值)。email 白名单**强校验**:`x@exploit-<id>.exploit-server.net` -> 3119B `Invalid email address` ⇒ 无「读自己的确认信」通路。
2. **确认腿 = POST**:`users.js` 的 `confirmEmail()` 从 URL query 造 `POST /confirm?<query>`。`GET /confirm?token=<任意值>` 一律 200(2839B,digest fa1c51e1e8659475,不校验 token),只是渲染页。
3. **空 token 被 app 拒绝**:`POST /confirm?token=` 与 `token=%20`、`token=+` -> **403 `"Forbidden"`**;`token=0`/`token=1` -> 400 `"Incorrect token: …"`;无 token -> 400 `"Missing parameter: token"`;`token=%00` -> 500。
4. **`token[]=` 绕过 403** 但绑定的是 `''`:响应 400 `"Incorrect token: Array"`(`token[a]=`、`token[0]=` 同)。批50 的 500「placeholder 1 vs 2 参数」说明该端点是参数化 `WHERE token = ?` 且把数组铺开绑定。
5. **登录**:`POST /login`(csrf 与注册同一个)未确认账号 -> 200 3801B `Invalid username or password`(无 302);缺 csrf 的对照腿因 CL 写错给 500,不作数。
6. **存在性检查是 TOCTOU**:同一用户名 8 个并发 `POST /register` -> **4× 2636(新 INSERT)+ 4× 3142(DUP)**。
7. **读侧播撒失败**:4 worker × 25ms × 9s 对 `POST /confirm?token[]=` 只发出 36 条(全 400),期间前台并发 4 个注册窗口 -> `hits=0`。
8. **可见性探针纪律**(批50 立,批51 复核):`/register` 自身即写者,不能当可见性探针;`/login` 被确认门挡、`/confirm` 只泄查询形态 ⇒ 该 lab 的 HTTP 层**没有**非变异读路径(见 [[visibility-probe-must-be-non-mutating]])。

## 判读与下一步

- 「半构造行 token = 空串」与 `WHERE token = ?` 的组合**被 403 证伪**(输入侧空值直接拒绝,数组形态也绑不到 NULL)。
- 未决面收窄为:被后置写入的列到底是哪一列(token 之外?email?confirmed 标志?),以及该列在窗口内的**可匹配形态**(NULL 不可由 query 到达 ⇒ 需要能到达 NULL 的信道或换观测原语)。
- 复攻建议:先找「非变异、能区分半构造行」的读原语(如把该行渲染出来的页/错误码差分),再谈窗口;单靠加大 confirm 量已被两批否证。

## 复现命令

```
http_dump <inst>/register --out reg.html        # csrf + phpsessionid(注意:http_session 偶发挂死,http_dump 稳)
raw_matrix '{"host":"<inst>","variants":[{"name":"regexp","line":"POST /register HTTP/1.1","headers":["Cookie: phpsessionid=…","Content-Type: application/x-www-form-urlencoded","Content-Length: 95"],"body":"csrf=…&username=u1&email=u1@ginandjuice.shop&password=pw12345678"}]}'
race_send <inst>/register --method POST --n 8 --form csrf=… --form username=rr1 --form email=rr1@ginandjuice.shop --form password=… --header 'Cookie: phpsessionid=…'
race_spread '<inst>/confirm?token%5B%5D=' --method POST --window-ms 9000 --interval-ms 25 --workers 4
```

## 关系

- 族:[[race-conditions-family]];方法见 [[http-2-single-packet-race-burst-method]]、[[visibility-probe-must-be-non-mutating]]。
