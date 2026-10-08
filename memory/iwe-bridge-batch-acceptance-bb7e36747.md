---
metadata:
  node_type: memory
name: "iwe bridge batch acceptance bb7e36747"
description: "iwe 桥件批验收:1 过(三子命令 1.1.0 全过)、2 过(裸 --root 现秒败 exit2;批核 1.0.0 曾静默吞)、3 过(zvec 不回退,h2_burst/postex_dns_poll rank1);HEAD 会话中由 bb7e36747 推进至 01766d7fc"
last_updated: 2026-10-08T12:32:16+08:00
created: 2026-10-08T12:32:16+08:00
---

## 2026-10-08 — iwe 桥件批四台分工回归验收(实测台)

权威 AGENTS.md;全程走件(rs_execute/rs_search/find_files/sh_run),未改仓文件,未提交。iwe_cmd.rs 工作树 == HEAD(无 diff)。

### HEAD 在会话中移动(重要)
- 会话开始 HEAD = bb7e36747(feat: iwe_cmd 桥件,core),其间 HEAD 推进到 01766d7fc(fix: iwe_cmd bare --root fails fast; mutating flag in envelope — kimi F1+G2/G4/G5)。
- 物化副本 ~/.pi-rs/agent/rust-materialized/iwe_cmd.rs 起初滞后(1.0.0),HEAD 推进后重跑自动刷新为 1.1.0(12:31:35)。验收必须对**活物化副本**判读(信封头版本),不能只看源树。

### item 1 — 三子命令(过)
`iwe_cmd stats` / `["find","race"]` / `["tree"]` 在 1.1.0 下全 success=true,data.raw 非空(stats 163 文档/4083 节点;find 竞态族图;tree 全量文档),data.mutating=false,data.root=/mnt/wsl/repos/pi-rs/.pi-rs/knowledge。(1.0.0 时亦三路全过。)

### item 2 — 裸旗标边界(评审 F1 对照)
- 实测当前行为(HEAD 01766d7fc / 物化 1.1.0):`iwe_cmd stats --root` → **秒败**,`{"action":"iwe_cmd","error":"--root requires a value","success":false}`,exit 2。
- 批核 bb7e36747(物化 1.0.0)实测:裸 `--root` 被**静默吞**(`root = argv.get(i+1)` → None → 落缺省根),照跑 stats 返回 success —— 正是 F1 缺陷。
- 判读:**该秒败**。裸旗标无值是操作者 typo;静默落缺省会让原语悄悄打到错库而无任何信号(源码注释原话)。契约应为非零退出 + failure 信封,现已落地。

### item 3 — rs_search 回归(vendor 引擎不回退)(过)
- `rs_search "single packet race"` → rank1 h2_burst(debug head `#1 matchedBy=fts+vector …h2_burst.rs:37-142`);`rs_search "dns 消费件"` → rank1 postex_dns_poll(head `#1 …postex_dns_poll.rs:15-126`)。两次 search.call engine=zvec,无 search.zg-fail(未回退 substring)。

### 复用经验
物化件(~/..rust-materialized)可能滞后已提交源;HEAD 可被并发会话推进。实测台判读锚点=rs_execute 信封头的件版本 + 实际行为,而非仓内源树版本;并发会话下先记录起始/结束 HEAD。

