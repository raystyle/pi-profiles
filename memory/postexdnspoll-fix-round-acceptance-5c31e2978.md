---
metadata:
  node_type: memory
name: "postex_dns_poll fix-round acceptance 5c31e2978"
description: "fix 轮 5c31e2978(postex_dns_poll v1.1.0)自省验收 2/2 过: selftest 九断言全绿(含 wire_case/zone_anchor_reject/resend_dedup), 缺 --mailbox 秒败 usage+exit2, 阳性对照(补 mailbox)走帧等待而非 usage"
last_updated: 2026-10-08T04:28:06+08:00
created: 2026-10-08T04:28:06+08:00
---

## 2026-10-08T05:0x — fix round 5c31e2978 (postex_dns_poll v1.1.0) self-inspection acceptance

权威: AGENTS.md。结论: 2/2 过。冷会话实跑,未改文件,未提交。HEAD = 5c31e2978
("fix: zone anchor, non-blocking pump, uplink confirmation (kimi F1-F3 + G1-G4)")。

- 件面(过):`rs_execute postex_dns_poll --selftest` → `data.selftest=="ok"`,
  九断言全 true: roundtrip / missing_shard_none / **wire_case** / **zone_anchor_reject** /
  **resend_dedup** / seq_split / terminal / b32_decode / sanitize。三个新增断言均在列且绿。
- 行为面(过):zone/session/binary 齐备但缺 --mailbox → 信封 `error=="usage"`、
  进程 **exit code 2**(rs_execute 尾部显式 "exited with code 2"),next_suggestion
  即 usage 行;符合 F/G2 的 mailbox 必需化秒败。
- 阳性对照(补):同参数补 `--mailbox /tmp/does-not-exist-mailbox.jsonl --timeout-ms 5000`
  → 不再打 usage,走到帧等待截止:`error=="no complete frame for session s1 within 5000ms"`、
  exit 1,且 stderr 每次轮询有 "mailbox read failed: No such file or directory"(G2 空转留因),
  证明 usage 分支确由 mailbox 缺失单独触发,而非解析巧合。

关键实现点(备查):usage 判定为 `zone.is_empty() || session.is_empty() || binary.is_empty() || mailbox.is_none()`;
mailbox 解析为 `Option<String>`(而非空串默认),故"给了 key 没给值"也计缺失。

