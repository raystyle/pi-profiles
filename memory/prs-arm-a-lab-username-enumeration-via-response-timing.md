---
metadata:
  node_type: memory
name: "PRS arm A lab-username-enumeration-via-response-timing"
description: "Arm A solve: timing-based username enum (announcements/monitoring) via new timing_enum piece with per-request X-Forwarded-For rotation; two oracle defects found and fixed"
last_updated: 2026-10-09T08:10:28+08:00
created: 2026-10-09T08:10:28+08:00
---


题面路径 canonical /web-security/authentication/password-based/lab-username-enumeration-via-response-timing;range_launch launch-url 一次起实例(reused:false),jar /tmp/cj1.json;解到 congrats。

机制与判据
- 登录表单 /login 只有 username/password(无 csrf);失败页文案是 `Invalid username or password.`(不是 "Incorrect password" - 按后者判命中会全量假阳性)。
- 时序差:对存在的用户名用 100 字符超长口令投递,中位耗时 ~1.44-1.51s;不存在用户名 ~0.95-1.05s。信号 ~+420ms,必用 madian 多次。
- 按 IP 计的爆破保护:XFF 轮换下 ~500 次投递 block 计数 0,连跑无封禁。

枚举
- 候选表取自 portswigger.net/web-security/authentication/auth-lab-usernames(100 名)与 auth-lab-passwords(100 口令),下载页 <code> 块即正文。
- 单趟 100 名 sweep 有漂移(基线从 1.5s 滑到 0.93s),排名靠 median 会误判;修正做法是拿少量候选 + wiener(已知有效,1440ms)+ 两个编造名(对照 ~960ms)在同一个短窗口内重测:announcements 1483ms 与 wiener 同级,判定有效。
- 结果:有效用户名 announcements,口令 monitoring(来自候选表),登录 302 Location /my-account?id=announcements + session cookie。

收口
- /my-account 返回 is-solved + "Congratulations, you solved the lab!" + "Your username is: announcements";banner_verdict solved:true。

新件 timing_enum v1.0.0(项目层,.pi-rs/rust-scripts/timing_enum.rs)
- users 模式:候选名 × --reps 次 POST(超长口令,默认 100 个 A),逐请求轮换 X-Forwarded-For(10.x.x.x 递推),出中位耗时排名;brute 模式:候选口令扫描 + 命中回写 jar + --confirm-path 复验。
- 两个自省抓到的缺陷已修:①命中判据从"缺 fail-marker"改为 3xx 或(缺 fail-marker 且确有 Set-Cookie),否则错 marker 会伪造命中;②账户页复验改成正向标记("Your username is" / "/logout")- 账户页自身含 class="login-form",用它的缺失会恒 false。

