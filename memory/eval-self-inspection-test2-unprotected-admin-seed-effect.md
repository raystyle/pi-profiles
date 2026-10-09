---
metadata:
  node_type: memory
name: "Eval Self-Inspection Test2 Unprotected Admin Seed Effect"
description: "题2 自省台:P13 非直达(r12/r4)+ 件组合不含直取 robots.txt 的件(http_session 未命名);range_launch 信封无复用信号,实测同 id 二次 launch 回同实例(评测面污染级)"
last_updated: 2026-10-08T15:13:29+08:00
created: 2026-10-08T15:13:29+08:00
---

## 2026-10-08 pi 自省台(seed 实效面)题2 @ab232f2f1,lab-unprotected-admin-functionality

评测臂现状:practice/eval/lab-unprotected-admin-functionality 仅 A 臂(A-2 @15:01、A-2-tainted @14:57,同 baseline .wt-A @0250f4120,dataTree 无模式层);无 B 臂。

### 1. P13 披露面(robots 子型)检索与件面 — 缺
seed 根 universe=64,iwe find --lexical(=knowledge 查):
- 「robots 披露 禁扫」→ disclosure-family rank 1/25、access-control-family rank 4/25、tactical-patterns(P13 所在)**rank 12/25**。
- 「disallow 侦察」→ tactical-patterns **rank 4/21**、access-control-family 15/21、disclosure-family 16/21。
=> P13 两条机器人措辞都非直达;直达的是族页(disclosure-family/access-control-family)。
P13 文本:机制原理写了「爬虫声明文件」(robots 覆盖到了),但件组合只写 `page_read`/`web_fetch` + `text_grep` —— page_read 是 PortSwigger 题页读取件(剥 solution),web_fetch 是 CDP 渲染件,无一直接取实例 `/robots.txt`;实测解题件是 `http_session get <instance>/robots.txt`(A-2 臂第 3 调用),P13 未命名。robots 子型的覆盖实际落在 access-control-family 的「未认证面 | 直访未防护路径;robots 与 JS 泄露路径」行。

### 2. range_launch 实例复用检测 — 缺(评测面污染级)
件版本 1.1.0;成功信封字段固定:`{lab_id, launch_url, final_url, status, instance_url, oidc_form_submitted}`,无任何复用字段。
实测(2026-10-08,lab id EFB3CD7D…8DE2,--jar /tmp/cj1.json):
- launch #1 → instance_url `0a4900f70450d7c880b58afa00760082`,status 200。
- launch #2(同 id,紧随)→ **同一** `0a4900f70450d7c880b58afa00760082`,信封形状逐字段相同。
=> PortSwigger 在实例存活期内对同 lab id 复用实例;信封无法区分「新起」与「复用」。
历史污染已在评测里发生:A-2 臂同一次 run 内 launch 两次(调用 2 与 6)都回 `0a24001a048ef20480b9b7dc0014004a`;A-2-tainted 亦落到同实例。
修法建议:按 lab_id 落台账(上次 instance_url + 时间戳),回 `reused`/`previous_instance_url`;或 launch 后探 `/` 回 `is_solved`,让 runner 直接弃臂(否则 B 臂可能继承已解状态,污染 A/B 对照)。
副作用:本次实测新起实例 `0a4900f70450d7c880b58afa00760082`(约 1h 存活),后续题2 复跑会复用它。

