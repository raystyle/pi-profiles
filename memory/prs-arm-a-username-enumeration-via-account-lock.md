---
metadata:
  node_type: memory
name: "PRS arm A username-enumeration-via-account-lock"
description: "arm A 基线:lab-username-enumeration-via-account-lock 冷实例一次通过(vagrant/superman),新件 acct_lock_enum v1.3.0;锁定 oracle = lock 提示只对存在账号出现、锁内正确口令给纯登录页,会话须等 60s 锁过期"
last_updated: 2026-10-08T20:50:39+08:00
created: 2026-10-08T20:50:39+08:00
---

臂 A 基线,题 `lab-username-enumeration-via-account-lock`,实例 0abe003a…(range_launch reused:false),一次通过 solved(banner solved:true)。

关键步:page_read 取 widget-lab-id → range_launch --jar /tmp/cj1.json → 官方候选表(usernames/passwords)经 http_session+doc_read 取回(各 100 条,md 第 105 行) → 新件 `acct_lock_enum` 枚举(100 名 ×5 次错口令):不存在名恒 "Invalid username or password",`vagrant` 第 4 次起出 "too many incorrect login attempts" ⇒ 有效用户名 → 爆破腿第 26 条 `superman` 返回既非 lock 非 invalid ⇒ 口令 → 等 65s 重放 → 302 /my-account?id=vagrant + 新 session → 账户页 "Your username is: vagrant"。

新件 `acct_lock_enum` v1.3.0(`.pi-rs/rust-scripts/`,含 selftest):枚举腿(`--users` 重复投递)+ 爆破腿(`--user`/`--passwords`)+ 命中后锁等待复验(`--lock-wait-secs`,默认 65)。catalog 已 regen(无 diff,项目件不入内置 catalog)。

本题机制(实测,可复用于同类锁定题):
- 锁定只对**存在**的账号触发 ⇒ lock 提示本身是用户名枚举 oracle;不存在名永不 lock。
- 锁定期内:错误口令 → lock 提示;正确口令 → 纯登录页(200,无提示),但**不发认证 session** ⇒ 锁内即可完成口令爆破,会话须等锁过期(实测约 60s)。
- 终判据必须是 `/my-account` 正文含 "Your username is:";未登录时 /my-account 是 302→/login(跟随仍 200),只看状态码会误判。

实录:`.pi-rs/knowledge/records/lab-username-enumeration-via-account-lock.md`(≤40 行)。未 git 提交。

