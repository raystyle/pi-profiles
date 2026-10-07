---
title: 竞态族:单包并发与状态窗口
---

# 竞态族:单包并发与状态窗口

同一类型漏洞:校验与状态写入之间的窗口被并发请求穿过,产生限速超额、
重复兑换、错投、半构造对象等。共性成因是「读-判-写」非原子,且 HTTP/1.1
多连接难以真正同刻到达;族内解法收敛到 **HTTP/2 单包齐发**(single-packet attack)。

写侧与读侧的节律不同:写侧(register/INSERT)要齐发,拼的就是同刻窗口;
读侧(confirm/SELECT)在窗口可达时反而要**播撒**(窗口总时长内以小间隔连续
发,而不是一个瞬间齐发),因为读侧要的是「撞上半构造行的任一时刻」,齐发
只在窗口起点撞一次。判据:写侧成功数证窗口可达后,读侧若齐发全 miss,
下一手是 spread 不是加量。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| 单端点竞态 | 同端点多请求、会话内单令牌 | 齐发改邮箱,争会话令牌最后写者 | [[lab-race-conditions-single-endpoint]] |
| 多端点竞态 | 校验与结算各在一端点 | 结算窗口塞入夹克(需单包) | [[lab-race-conditions-multi-endpoint]] |
| 时间敏感令牌 | token 随时间/按用户变 | 由已知 token 反推派生式(含 salt 难) | [[lab-race-conditions-exploiting-time-sensitive-vulnerabilities]] |
| 部分构造 | 注册与确认分离 | 穿过 INSERT→UPDATE 窗口用半构造对象(实测 `WHERE token = ?`;真单包 4 种排布共 ~70 confirm 全 miss ⇒ 半构造行对确认腿不可见/不匹配,NULL 或未提交嫌疑最大) | [[lab-race-conditions-partial-construction]] |

## 共性

1. 先证窗口存在:并发齐发能造出**错投/超额**等可观测证据,再收窄到目标动作。
2. 同刻到达是核心:HTTP/1.1 双连接 + JS 同 tick fetch 常落在窗口之后;
   正解是 HTTP/2 单包(参见 [[h2-smuggling-family]] 的帧工具)。
3. 会话/串行约束:PHP 靶场会话级串行、购物车按会话,需换不同/无会话并发
   (`race_send --no-cookie-b`)。**批42 反例**:部分构造题同 session 并发全部成功(存在性检查 TOCTOU),会话并未串行化。

## 工具面

- `race_send`:每 worker 预热连接、Barrier 齐发,`--url2`(异构端点)、`--stagger-ms`、`--out-dir`。
- `race_email`:改邮箱 A/B 齐发 + 读收件箱抽 token + confirm 驱动。
- `sha_brute`:时间派生式暴力(整数/毫秒/微秒/uniqid/日期 × user/email/pw)全未命中。

## 部分构造:硬证据与陷阱

- **同 session 真并发**:单包 `h2_burst` 里同一 session 连发 3 个相同 `POST /register` → **三条全成功** ⇒ 存在性检查是 TOCTOU,且会话**不**串行化。遇到“同 session 一定被锁死”的判断先证伪再下结论。
- **DB 层报错可挖查询形态**:数组参数铺开会让占位符计数对不上 —— `POST /confirm?token[][]=&token[]=` → 500
  `The query does not contain the correct number of placeholders 1 for the number of arguments passed 2`
  ⇒ 该端点是参数化 `SELECT … WHERE token = ?`,且把 token 数组当 bind 参数铺开(1 元素可用、≥2 报错)。这是一条**通用手法**:拿 `k[][]=` + `k[]=` 两元素去照出“1 个占位符”的参数化查询。
- **NULL 不是可绑值**:`WHERE token = ?` 永远匹配不到 NULL,而 PHP 里能与 NULL 相等的**空数组**在 query 解析器下不可达(`k[]=` 只得 `[""]`)
  ⇒ 若半构造态是 NULL,「绑空串穿窗口」这条就永不可中;应当转向“让半构造行可见/可读”的其它原语(如把 token 回显到响应/日志的端点)。
- **批42R 追加否证**:真单包(`single_packet_likely: true`,8-10 流)下四种排布(`2 reg+6 conf`、确认腿不带 cookie 以排除 session 锁、`4×(reg+4 conf)` 每 reg 换新用户名、大包 `10 reg+50 conf`)累计 ~70 个 `token[]=` 确认全回 `Incorrect token: Array`;
  而同包 10 个同名 register 前 5 个成功 ⇒ **register 的竞态窗口对本连接可见、对 confirm 的 SELECT 不可见**。
  未决面收窄为:INSERT 是否未提交的事务可见(隔离级别),或 token 列默认值是 NULL 而非 `""`。单包预算 ≈1400B 时一轮放不下成文写本的“20-40 regs × 50-60 confirms 反复”,续攻须先把「多轮单包 burst 循环」落成件。

- 判定锚点:竞态动作实际生效(邮箱被改、以非预期价格下单、限速被绕过);
  仅「并发完成」不算解。
- 4 题现状:单端点/多端点/时间敏感/部分构造均 **stuck**——均已证窗口与错投,
  卡在 HTTP/1.1 无法对齐单包窗口;下一步候选取 h2 同包件。

## 相关族

- 单包帧工具与 h2 面见 [[h2-smuggling-family]];方法论:web-vuln-methods(seed 层,按名引用)。


## 部分构造的可见性 oracle 与换维纪律(批49 蒸馏)

- **可见性 oracle 用响应体大小类**:半构造行"谁还能看见"的判据是页面二值
  (新 INSERT 页 vs 重复键页,实测 2636B vs 3142B),确定性、不依赖时间;
  race_spread --class 分桶即为此。注意:同刻齐发的存在性检查都发生在任何
  INSERT 完成之前,与可见性无关——可见性判据必须是窗口内错峰播撒的探针。
- **时序维度穷尽后换的是 oracle 维度,不是样本量**:齐发/播撒/h2 单包三种
  并发模型全 miss 后再加量是浪费;error-based 只泄查询形态(placeholder
  计数),时间差不可用(单状态无命中可标定,抖动>信号)——三条负面记死。
