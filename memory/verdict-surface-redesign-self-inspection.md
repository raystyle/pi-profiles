---
metadata:
  node_type: memory
name: "Verdict surface redesign self-inspection"
description: "判定面重设计自省:item1 半缺(llm 真锚是 mixed 行;判据还误中 user 派单词),item2 缺(prefilled 助文本距 EOF 9 行>8),item3 缺(trace.sh _sid 漏 -N/-A3 后缀);新件 transcript_judge"
last_updated: 2026-10-10T21:42:15+08:00
created: 2026-10-10T21:42:15+08:00
---

## 2026-10-10 判定面重设计自省(verdict-face-genealogy #7 prefilled 鹦鹉案)

三靶:`practice/eval/lab-prefilled-form-input/A-1/transcript.jsonl`(143 行)、`lab-exploiting-insecure-output-handling-in-llms/A-1/transcript.jsonl`(102 行)。新件 `.pi-rs/rust-scripts/transcript_judge.rs`(1.0.0,逐行分块 + 标记落点 + 距 EOF)。

**item1 角色判别判据 = 半缺。** 鹦鹉侧成立:prefilled 142 行(assistant,types=[thinking,toolCall],raw 含 `thinking` 不含 `"type":"text"`,thinking 内 solved/congrats/终态)、llm 94/96 行(同形)。但真锚侧不成立:llm 的 assistant 末条锚(102 行,末行)是 **mixed** 行——同一条同时含 thinking 块与 text 块(raw 两项皆真),按「行内无 thinking」会被丢掉。且「含 `"type":"text"` 且不含 thinking」两字段也命中**用户派单词**(两档 5 行,role=user,text 内含 终态 模板,即 #2 模板回声复发)。收敛形须下沉到**块级+角色级**:锚 = 同一行内 role=assistant 的 text 块(兄弟 thinking 块不参与),或 toolResult 的结构化回执;不可用行级负向。

**item2 尾窗 = 缺(差 1 行)。** llm 真锚 102 行 dist=0(窗内);prefilled **无 assistant 终态文本锚**(进程在 143 行 toolResult 处收档,末条 text 是 134 行,dist=9,已出 8 行窗);落判证据是 toolResult text 行(141/143,dist 2/0)。故 tail -8 仅在「锚允许取工具回执」时够;按「assistant 末条 text」定义则 prefilled 漏 1 行,需扩到 tail -12 或改结构化回执判定(方向 #4)。

**item3 trace.sh 子账 sid 漏配 = 缺(已复现)。** `run.sh` 写 session-id `eval-$LAB-$ARM-$N`(如 `-A-1`);trace.sh 只推 `eval-{lab}-{arm}`(无 N),且 glob 写死 `/{_sid}/tasks`,故两类自定义 id 均失配。实测 /tmp/pi-rs-subagents-1001/mnt-wsl-repos-pi-rs/ 下仅两个 eval-* 目录:`eval-lab-host-header-ssrf-via-flawed-request-parsing-A3`、`eval-lab-prototype-pollution-client-side-prototype-pollution-via-browser-apis-A`(其余为 UUID 与 s1-* 手工会话)。exact 形 find:host-header-A=0(漏)、browser-apis-A=1(撞对)。修正形 = 前缀通配 `/tmp/pi-rs-subagents-*/**/{_sid}*/tasks/*.output`:host-header-A*=3、browser-apis-A*=1。未改 trace.sh(验证轮不落生产改动);两靶运行本身无子代理目录。

纪律:全程走件;未 git 提交。

