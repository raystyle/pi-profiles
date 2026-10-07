---
title: 可见性探针必须非变异(visibility probe must be non-mutating)
---

# 可见性探针必须非变异(visibility probe must be non-mutating)

判断「半构造行/中间态对象对其它连接是否可见」时,探针**自己不能是写者**,否则实验被污染。

## 反例(partial-construction 题)

prescript 曾建议用 `/register` 做可见性探针(同 username,按体长类 NEW:2636 / DUP:3142 判读),
并配 write 侧齐发同名注册造窗口。实测:

- read 侧 `race_spread /register` **sent 153 / DUP 149 / NEW 4**;
- write 侧 `race_send 20 同名注册` **20/20 DUP**;
- **原因**:`/register` 探针第一次成功 INSERT 并提交后,它自己就把该 username 建好了 ⇒ 之后全是 DUP。
  读出的"DUP 命中"只证明**探针自己的行**可见,与 write 侧半构造行无关。

⇒ 该实验的类分布**不能**在假设①(未提交事务)与②(行可见)之间二选一。纪律:**先问探针会不会写**。

## 干净探针的找法

1. 找**只读**端面:`/login`、`/confirm`、`objref` 读、任何回显用户表的页。
2. 若只有写端面:找「过完存在性检查、却在 INSERT 前因其它校验被挡」的**非变异档位**
   (本实例校验顺序 = email → 存在性 → INSERT,无此档;`email` 非法会**先**短路,拿不到存在性信息)。
3. 若两者都无 ⇒ 该 lab 在 HTTP 层无可见性探针,直接记死,别用被污染的读数下结论。

## 附带判读

- 注册行**提交很快**(同波并发里后到者已能看见先到的 INSERT)⇒ 长事务未提交(假设①)不像成立。
- 「时序维度穷尽后换的是 oracle 维度,不是样本量」:先确认 oracle 非变异,再谈换维度。

## 关系

族 [[race-conditions-family]]、[[hunter-differential-method]];实例 [[lab-race-conditions-partial-construction]]。

## Links

- evidences: [[race-conditions-family]]

- evidences: [[lab-race-conditions-partial-construction]]
