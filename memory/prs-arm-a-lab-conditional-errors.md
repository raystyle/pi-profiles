---
metadata:
  node_type: memory
name: "PRS arm A lab-conditional-errors"
description: "arm A 基线:lab-conditional-errors 冷实例解出 - TrackingId 条件报错盲注取 administrator 口令 20 位,两处坑=件瞬断造成假终止(须 LENGTH 复核)与候选字符 `;`/`%` 塞进 cookie 值造成的 500 假阳性"
last_updated: 2026-10-09T05:56:57+08:00
created: 2026-10-09T05:56:57+08:00
---

## 2026-10-09 arm A: lab-conditional-errors solved (Oracle 条件报错盲注)

- 实例: range_launch launch-url /web-security/sql-injection/blind/lab-conditional-errors(reused:false)→
  0ac400da03cf1c1e805f123200f700e1,jar /tmp/cj1.json(已带 TrackingId=O3WmCmmKbTTB3qHP)。
- oracle 面: TrackingId cookie 注入,真条件 → 500(lab 错误页 2330B,含 `Internal Server Error`),
  假条件 → 200(商城页 11433B)。DBMS 判为 Oracle:`'||(SELECT CASE WHEN (<cond>) THEN TO_CHAR(1/0) ELSE '' END FROM dual)||'`
  真/假两发差分(1=1 500,1=2 200)一次定位。
- 取数: 件 blind_oracle,`--template "...SUBSTR((SELECT password FROM users WHERE username='administrator'),{I},1))='{C}'..."`,
  `--true 'Internal Server Error' --place cookie:TrackingId --jar <无 TrackingId 的 jar>`。
  注意: jar 里若已有 TrackingId,件会再追加一个同名 cookie(重复),故用仅 session 或空的 jar(/tmp/cj_blind.json={})。
- 坑1(件面): 单发请求瞬时失败 = 该字符假阴性,件无重试 ⇒ 该位判为 None → 提前 terminated。
  首轮 12 线程在 16 位踩到,得到 15 位假长度(前缀 1-15 正确);用 LENGTH 数值探针才发现 >15;
  同 charset 降线程重跑(threads 4)一次拿全 20 位。教训: 件给出的 terminated 不能当作密码长度,必须用 LENGTH(oracle) 复核。
- 坑2(件面/信道): 把候选字符直接塞进 cookie 值里时,`;` 会截断 Cookie 头、`%` 触发解码异常,
  两者都让请求自身报错 → 500 假阳性(扫描时先撞到 `;`,去掉后又撞到 `%`)。
  修正: 换成不落候选字符的数值探针 `ASCII(SUBSTR(password,16,1)) BETWEEN ...` 分带,确认 16 位其实是小写字母(97-122)。
- 结果: 口令 busojpshfk3srb5ndfgo(20 位,LENGTH=20 复证);/login 取 csrf → POST administrator 登录 → /my-account?id=administrator
  首访报 notsolved 但 banner_verdict 复读得 `Congratulations, you solved the lab!`(solved_class=true)。
- 纪律: 未读题解/records/族注;HTTP 全走件;未 git 提交。

