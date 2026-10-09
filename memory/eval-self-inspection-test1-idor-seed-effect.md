---
metadata:
  node_type: memory
name: "Eval Self-Inspection Test1 IDOR Seed Effect"
description: "题1 IDOR 自省台:族页直达(r1)但 P 层非直达;「两臂没引用 P11」是臂树覆盖假象(A/B-1 树无 P11),接线缺口未证;强制族判型不会更短,主因是派单词只点名 rs_search"
last_updated: 2026-10-08T14:49:55+08:00
created: 2026-10-08T14:49:55+08:00
---

## 2026-10-08 pi 自省台(seed 实效面)@ab232f2f1,题1 lab-insecure-direct-object-references

setup:practice/eval/lab-insecure-direct-object-references 臂 = A-2/A-3(baseline .wt-A @0250f4120,无模式层)、B-1(@414dd48a4,seed v1=P1-P10,无 P11)、B-2(@1e2f3c0c,含 ab232f2f1/P11,但仅 16 行 8 调用、未解、零 knowledge)。派单词(prompt.txt)只写「range_launch 起实例,rs_search 找件」,从不提 knowledge/族判型。

检索面实测(seed 根 universe=64,iwe find --lexical=knowledge 查实现):
- 「权限边界 差分」→ tactical-patterns rank 4/25,access-control-family rank 21/25。
- 「对象引用 越权」→ access-control-family rank 1/26,tactical-patterns rank 10/26。
=> 族页对 IDOR 措辞直达;P 层两条查询都非直达,且族页与 P 层无法同榜召回。

接线缺口命题不成立(是臂树覆盖假象):A-2/A-3 起步第 2 步就查族判型(query "insecure direct object references IDOR access control [lab method]" + get_knowledge key=access-control-family,族页含本题原形「越权读聊天记录取他人口令并登录其账户」);B-1 解题期零知识调用(首次 knowledge 在 banner_verdict 之后,只用于写 record);A 臂树无模式层、B-1 树早于 P11 → 两臂无法引用 P11 是树里没有。含 P11 的 B-2 未跑完,故「检索通/解题没用上」尚无有效样本。

假设检验(强制首查族判型是否更短):不会。最短臂 B-1 55 行/27 调用,解题期零知识,靠 rs_search「object reference id enumeration」→objref_scan 直达;做了族判型的 A-3 仍 95 行/31 rs_execute(range_launch×11 + 会话绕行)。归因排序:派单词(主)> seed 措辞(P11 触发词「客户端可控维」不覆盖对象引用枚举子型,P11 件组合 http_dump/reqseq/cred_matrix 不含 objref_scan)> 工具面(find 单查询族↔P 不互达)。

待办:以 B 臂在 ab232f2f1 上跑一次完整解题(≥1 次知识判型查询),才能证/否接线缺口;若仍不引用,再考虑 prompt 加一句族判型或给 P 层加 IDOR 子型/件名。

