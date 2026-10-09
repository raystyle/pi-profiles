---
metadata:
  node_type: memory
name: "Seed v2 Retrieval Acceptance ab232f2f1"
description: "seed v2 检索面验收 @ab232f2f1:3/3 过 - 新增/变体查询全命中 tactical-patterns(唯一偏弱:换信道 rank 9/26),seed P 计数 24、全局库副本同文"
last_updated: 2026-10-08T13:27:37+08:00
created: 2026-10-08T13:27:37+08:00
---

## 2026-10-08T13:5x batch: seed v2 检索面验收 @ab232f2f1(HEAD,工作区仅 .pi-rs 子模块 M + practice/ 未跟踪)

判据与实测(bundled seed 根 = packages/rs-agent/src/extensions/knowledge/seed;iwe find --lexical = knowledge 查的实现,source 已核 actionArgs find → ["find","--lexical",q,"-f","keys"]):
- seed 文档全集 universe=64。
- 新增召回 4/4 命中 tactical-patterns:权限边界 差分→rank 4/25(P11)、披露面 侦察→rank 2/25(P13)、窗宽 配型→rank 1/25(P23)、免手势 引爆→rank 3/26(P24)。查询词逐字见于目标节(行 174/189/264/272)。
- 变体召回 2/2 命中:末字节 同刻→rank 2/24(P2 变体注「末字节同步粒度」/「同刻」)、换信道→rank 9/26(P5 标题「无回显换信道」,本批最弱腿)。
- seed 全文计数:^### P = 24 条(P1..P24),P11..P24 = 14 条;抽查 4 条标题在场(P11 行174 / P13 行189 / P23 行264 / P24 行272)。
- 全局库副本 /home/ray/.pi-rs/agent/knowledge/tactical-patterns.md 与 seed 字节相同(diff -q identical, mtime 13:25),故 tier 间检索一致。

结论:3/3 项过(变体项「换信道」rank 9 记为偏弱但命中)。无缺项。

工具口径:排名取证用 sh_run 跑 iwe(在 PATH: fnm_multishells/.../bin/iwe)+ nl/grep(capped CTA 提醒无碍);全文计数用 text_grep;层级一致性用 diff。未改任何被验文件、未 git 提交。

