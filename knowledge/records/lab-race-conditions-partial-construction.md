---
title: lab-race-conditions-partial-construction
---

# lab-race-conditions-partial-construction

> evidences: [[race-conditions-family]]

- 题面:Partial construction race;绕邮箱验证建号 → 登录 → 删 carlos。
- 实例(批48):https://0a30003a0313a847801d49bf00ef0088.web-security-academy.net
- 状态:**stuck**(读侧播撒件已上,仍零命中 ⇒ "样本不够"这一假设被否证)

## 新证据(读侧窗口播撒)

1. **write 侧窗口可达(复证)**:`race_send <inst>/register --a 20`(20 并发同名注册)→ 状态全 200,其中 **4-5 个是新 INSERT**(2636B),其余是重复用户名页(3142B)。
2. **read 侧播撒(新件 `race_spread`,读侧专用)**:
   - 第一轮 `--window-ms 3000 --interval-ms 40 --workers 4`:只发出 **12** 个 confirm(全 400)⇒ 实测 `/confirm?token[]=` **每次约 1s**,worker 是串行的 ⇒ 间隔参数不等于采样率,worker 数才是;
   - 第二轮 `--window-ms 12000 --interval-ms 10 --workers 16`:发出 **151** 个 confirm,窗口覆盖一整轮注册(含 5 个新 INSERT)⇒ **151/151 = 400**,`hits:0`。
3. **时延侧写**:20 并发 `/register` 整批耗时 **11.5s**(5 个走 INSERT 路径)⇒ 若 INSERT 与 UPDATE 之间夹着慢动作(发确认邮件),窗口应是**秒级**;151 个均匀样本覆盖 12s 仍全 miss。
4. 累计(批46 齐发 260 + 批48 播撒 151 + 批46 早期 ~70)≈ **480 次**空 token confirm,全部 `400 "Incorrect token: Array"`。

⇒ 结论:**窗口不是"采样不够"的问题**。在秒级窗口上均匀播撒都打不中,只剩两种解释:①INSERT 在显式事务里未提交,半构造行对其它连接不可见(而**同 session 的 confirm 会被 PHP session 锁串行化**,永远排在 register 之后);②token 列默认 NULL(则 `WHERE token = ?` 永不命中 `''`)。两者都不是时序问题,靠加样本无法突破。

## 未决面/下一步(需要换维度,而不是加样本)

- 设法**观测半构造行本身**:找能回显用户表的端点(注册后的确认页/账户页),在 register 进行中读一次,直接判定"行是否可见、token 是什么值";这一步能一次性区分①②。
- 若①成立,唯一出路是让 confirm **与 register 复用同一 DB 连接/事务**(HTTP 层不可达)⇒ 该 lab 在本 infra 结构性不可解;若②成立,则要找"非空但可预测"的半构造值。
- 件面:`race_spread` 的 `--hit-substr` 与状态分布已是正确判据;`race_send --stagger-ms` 的齐发形保留作对照。

## 新证据(批49:h2 单写齐发 = 最后的未试维度,仍全 miss)

`h2_burst`(一条 h2 连接、一次 write、所有请求作为独立 stream 同时到达):

1. 1 register + 25 `POST /confirm?token[]=`(带 jar)→ register **200/2636**(新 INSERT 成功),25 confirm **全 400 `"Incorrect token: Array"`**;`burst_bytes 3937`。
2. 同上,但**只有 register 带 `Cookie: phpsessionid=…`**(排除 PHP session 文件锁把 confirm 串行在 register 之后)→ 仍 25×400;`burst_bytes 2645`。
3. 2 register + 15 confirm(混 `token[]=` / `token[]` / `token=`):`token=` 一律 **403 `"Forbidden"`**(空标量被当「无 token」早挡),`token[]=` 与 `token[]` 一律 400 `Array`,零命中。

⇒ 载荷形状被独立验证是对的(与 writeup 的「token[]= 有效、空标量 token= → Forbidden」完全一致),**问题既不是采样也不是并发模型**:h2 单写齐发已覆盖 register 的整个处理时段,半构造行对**另一条连接/stream**始终不可见 ⇒ 只剩两种解释未排除:(a) INSERT 在未提交事务里,(b) token 列默认值不是 `''` 而 `= ''` 永不命中。累计约 **530 次**空 token confirm 零命中。判定仍 **stuck**(时序维度已彻底用尽,下一步只能换维度观测半构造行本身)。

## 复现命令

```
range_launch launch 36E4A9EA…1A4E90A9 --jar ~/.pi-rs/agent/chrome-jar.json
http_session get "https://<inst>/register" --jar <jar> --out /tmp/reg.html     # csrf
race_spread "https://<inst>/confirm?token[]=" --body '' --method POST --window-ms 12000 --workers 16 --hit-substr 302 --warmup   # 先起读侧(后台)
race_send "https://<inst>/register" --form csrf=<csrf> --form username=atk2 --form email=atk2@ginandjuice.shop \
  --form password=pw12345678 --method POST --a 20 --jar ~/.pi-rs/agent/chrome-jar.json                        # 再起写侧
```

## 关系

- 族:[[race-conditions-family]];方法侧见 [[http-2-single-packet-race-burst-method]]。
