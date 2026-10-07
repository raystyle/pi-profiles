---
title: lab-race-conditions-partial-construction
---

# lab-race-conditions-partial-construction

> evidences: [[race-conditions-family]]

- 题面:Partial construction race;绕邮箱验证建号 → 登录 → 删 carlos。
- 实例(批46):https://0a41001c036a6b0182d879ec007500ef.web-security-academy.net(exploit 邮箱 `exploit-0ad60030033e6bb682fc78f00143004a`,客户端 `/email`)
- 状态:**stuck**(多连接并发件把 4 种延时档位打穿,260 次 confirm 零命中;未决面收窄到"半构造行是否对其它连接可见")

## 新事实(推翻/量化)

1. **邮箱域名白名单**:注册非 `@ginandjuice.shop` 邮箱 → 页面 `Invalid email address`;只有 `@ginandjuice.shop` 能建号 ⇒ **确认邮件永远读不到**(email client 只显示发给 `*@exploit-…` 的邮件)⇒ 无法用"真 token"做控制实验,只能靠竞态。
2. **多连接并发件是正确原语**(`race_send --url2 <confirm> --body2 '' --a N --b M --stagger-ms X --no-cookie-b`,每 worker 一条预热连接 + 屏障同放):
   每轮 10-25 个同名 `/register` 里稳定有 **4-6 个新 INSERT 成功**(200/2636B),其余拿 3142B 重复用户名页 ⇒ **register 的窗口确实可达**(与批42R 的 h2 观测一致)。
3. **confirm 侧全 miss**:6 轮共 **260 次** `POST /confirm?token[]=`(CL 0、不带 cookie 以排除 PHP session 锁),`--stagger-ms` 取 0 / 60 / 80 / 250 / 350 / 800,burst 10-25 regs × 25-60 confirms,**100% `400 "Incorrect token: Array"`**。
4. **bind 语义复核**(新实例):`POST /confirm?token[][]=&token[]=` → 500(占位符 1 vs 参数 2)⇒ 数组确实被铺进 bind,`token[]=` 真的跑 `WHERE token = ''`;`?token=` → 403、`?token=hello` → 400 `"Incorrect token: hello"`。

## 未决面(收窄为二选一)

- ① INSERT 在显式事务里未提交(则其它连接的 confirm SELECT 永远看不到半构造行);② token 列默认 NULL(`= ''` 永不命中)。两者都能解释"register 窗口可见、confirm 全 miss"。
- 下一手候选:把 confirm 从"一个瞬间齐发"改成**在窗口上连续播撒**(现件只能给一个固定 stagger,缺"spread over window"件);或 h2 单包里把 confirm **交错插进** register 之间(批42R 试过 4×(reg,4×conf),未试紧交替)。

## 复现命令

```
range_launch launch 36E4A9EA…1A4E90A9 --jar ~/.pi-rs/agent/chrome-jar.json
http_session get "https://<inst>/register" --jar <jar> --out /tmp/reg.html     # 取 csrf + phpsessionid
race_send "https://<inst>/register" --form csrf=<csrf> --form username=atk1 --form email=atk1@ginandjuice.shop \
  --form password=pw12345678 --url2 "https://<inst>/confirm?token[]=" --body2 "" --method POST \
  --a 10 --b 25 --stagger-ms 60 --no-cookie-b --jar ~/.pi-rs/agent/chrome-jar.json
```

## 关系

- 族:[[race-conditions-family]];平台侧见 [[portswigger-platform-specifics]]。
