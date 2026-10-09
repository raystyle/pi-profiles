---
metadata:
  node_type: memory
name: "PRS lab solve IDOR access-control"
description: "IDOR 题冷会话重解一次通过:page_read→range_launch→objref_scan 命中 1.txt→读明文口令→carlos 登录→banner 判 solved;另记 range_launch 首次不带 jar 落 auth0、DNS 失败为瞬时两个坑"
last_updated: 2026-10-08T13:09:02+08:00
created: 2026-10-08T13:09:02+08:00
---


## 2026-10-08 — IDOR 题冷会话重解(件链路,一次通过)

题:`access-control/lab-insecure-direct-object-references`(lab_id `BF6B2F75...4D70857`),
实例 `0aa800440378c64d803a58ea002300de`,**solved**:`banner_verdict` 回
`solved:true` + `<h4>Congratulations, you solved the lab!</h4>`。

链路(全部走件,5 步零弯路):
1. `page_read` 取 lab_id → `cp /tmp/cj1.json /tmp/idor-jar.json` → `range_launch launch <id> --jar`。
   首次不带 jar 直接 launch 落到 auth0 并报 `login.portswigger.net` DNS 失败;换 app-session jar 即通
   (与 R1 记录的「发射钥匙 = portswigger.net .AspNetCore.CookiesC1/C2」一致)。
2. `objref_scan <inst>/download-transcript/FUZZ.txt --ids 1-6` → **只有 1.txt 命中**(200,520B);
   2-6 全 400 `"No transcript"`。即该模板的存活对象就是 1,不必先发消息造 transcript。
3. 1.txt 内容 = carlos 与客服 Hal Pline 的对话,明文口令在 `You: Ok so my password is <pw>`。
4. `http_session get /login` 取 csrf → `post /login`(carlos + 该口令)→ 302 `/my-account?id=carlos`,
   页面 `Your username is: carlos`。

要点/坑:
- 读 transcript **不需要登录**——对象引用本身就是缺失的授权;chat 组件是 websocket,与解题无关。
- `chrome_cookies` 依旧会覆盖 `chrome-jar.json`(R1 旧坑),别指望那个 jar;用 `/tmp/cj1.json`。
- 本次 login.portswigger.net 的 DNS 失败是**瞬时**的(getent 复核立即正常),不是会话失效。
- 知识面:`records/independent-idor` 已存在,只把 Reproduce 节改写成上面的「标准件链路」,
  未加批次/日期历史(知识记关系,历史归 memory)。

