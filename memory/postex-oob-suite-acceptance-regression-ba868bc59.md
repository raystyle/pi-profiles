---
metadata:
  node_type: memory
name: "Postex + oob-suite Acceptance Regression ba868bc59"
description: "复验 item 1 @2f9e3793b:ranker 稀释修复后 rs_search「后渗透套件 通道」仍只 postex_build 进前 6(1/5),索引已确认刷新,排序逐位不变"
last_updated: 2026-10-08T03:32:22+08:00
created: 2026-10-08T03:29:06+08:00
---

## 结论(HEAD ba868bc59,逐项)

1. **发现面 — 缺**。`rs_search "后渗透套件 通道"` 实测三次同序:dbg_serve, postex_build, oob_serve, tunnel_variant_scan, dbg_cli(信封 matches=6)。原始 zg 排序(limit 12,直跑 `~/repos/zvec-grep/rust/target/release/zg` over `~/.pi-rs/agent/pi-rs-script-mirror`)前 6 命中:dbg_serve×3, postex_build#3, oob_serve#4, tunnel_variant_scan#6 —— postex 五件只 postex_build 进前 6,postex_relay/http/dns/exec 连前 12 都没有。对照:`rs_search "postex"` 五件全进前 5(索引里有全部件),失败面是 CJK 混词查询的语义打分。debug.log 证 engine=zvec(非 substring 降级)。
2. **selftest 三件 — 过**。postex_build → selftest:ok,**7 个命名断言**(scaffold_toml/scaffold_wrap/content_hash/arch_parse/sdk_embedded/raw_delimiter_guard/braces_passthrough),非任务文本所说的 9;历史版本亦从未到 9(b2fc8a0fa=6, f0fa4debc=7)。postex_exec → selftest:ok(frame_roundtrip/eof_none)。postex_dns → selftest:ok(base32/shard_budget/terminal_marker/label_bounds 四 true)。
3. **HTTP 腿活测 — 过**。relay `9190`/workdir `/tmp/ph-relay`;playbook `/tmp/ph-pb.rs`(file::Write /tmp/ph-relay-marker + shell::Run id);postex_build 出 `ph-pb-x86_64-c7e21ab96a` 444184B;postex_http `ok:true steps:2 changed:2`,marker 落盘内容 `ph-relay-ok`(12B),首跑事件 hello+write-marker(changed)+id(changed);**重跑 changed:1,write-marker=ok(clean)** → 幂等成立。测毕 relay 已 kill(bg-99d7dabb killed after 48s,ps 零残留),marker 与 /tmp 产物已清。
4. **DNS 腿活测 — 过**。自有 zone `oob.dthack.io`(通配 → 47.131.34.33)。`postex_dns --zone oob.dthack.io --resolver 8.8.8.8 --frame-file /tmp/ph-start.json` → frame 48B,shards:2,confirmed:2,failed:0。
5. **oob_suite 面 — 过**。`oob_serve --selftest` → selftest:ok,jesc:true query_param:true tail_missing_empty:true。`rs_search "canary 收件台"` → oob_serve #2(进前 3)。
6. **泛化 — 过**。`rg "oob\.dthack\.io|47\.131" postex-suite/` 零命中(exit 1);zone 已参数化。

## 顺带观察(非验收项)
- `postex_http.rs` 物化时 rustc 告警 `unused_assignments`:`exit_code`(117/164 行)赋值后被覆盖 —— AGENTS.md 要求零告警,值得在后续修。
- `oob_serve.rs` 携带静态 lint 建议「no AgentResult envelope」(源级 lint),但 selftest 实际有信封。
- 本会话 rs_search 的 CJK 查询排序弱于 ASCII:`postex` 全中、`后渗透套件 通道` 仅 1/5,与历史 memory(f0fa4debc 记「丢 postex_exec」)一致但更糟。

纪律:全程走件;仅写 /tmp;未改仓、未 git 提交。relay 后台进程与 marker 已清。


## 2026-10-07

## 复验 item 1 @2f9e3793b(ranker 稀释修复)

commit `2f9e3793b`「fix: postex acceptance round - discovery heads carry the family, poll loop drops its warnings」只改 **4 件**(postex_dns/exec/http/relay 的 description 头加「后渗透套件 postex …(postex_<name>,通道族…)」),**postex_build.rs 未改**;catalog.json 同步。

实测 `rs_search "后渗透套件 通道"`(索引 settled 后复测,debug.log `search.call matches:6 engine:zvec ms:586 needsIndex:false`):
序位 1 dbg_serve / 2 postex_build / 3 oob_serve / 4 tunnel_variant_scan / 5 dbg_cli / 6 race_spread —— **postex 五件仅 postex_build 进前 6(1/5)**,http/relay/dns/exec 连返回集都没有。与修复前**逐位相同**(zg-raw bytes 均 1725)。

已排除 stale-index 假阴性:mirror 03:31:03 已同步新头;`.zvec-grep` 03:31:04 manifest + 03:32:03 embedding.index.*.proxima,113 个文件晚于 03:30,`zg --status` coverage 95/95 queue 0。即索引确含新文本,排序仍不动。

副作用观察:zg 的 FTS tokenizer=jieba,embedding=local/potion-code-16m-v2(256d)。对 `操作侧` 这类描述内已有 CJK 词能召回 postex_dns/relay,但新加词 `通道族` 召回不到那四件 —— 描述头的 CJK 新词对该 code 模型既不进 FTS 有效词表也不拉高向量分,故「加族名+自名」未改变该查询排序。 **[2026-10-08 更正]** 此论不完整:真根因是 zg Rust 切块器的头附着断裂(//! 头与首个符号之间隔着 use 时头整段不进索引,fts/vector 两腿全瞎),与词表无关;见 knowledge/rs-search-mirror-header-relocation.md 与 6b105f78d。ASCII 查询正常(`postex` 五件全进前 5;`postex relay channel events` 命中 http/relay/dns)。

判定:item 1 **缺**(不满足 ≥4/5 进前 6)。

