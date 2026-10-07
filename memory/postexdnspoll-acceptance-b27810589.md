---
metadata:
  node_type: memory
name: "postex_dns_poll Acceptance b27810589"
description: "postex_dns_poll 批自省验收 @b27810589:件面 selftest 6/6 过;发现面半缺 - zvec 引擎下「dns 消费件」「消费件」均不召回新件(仅「postex dns」/exact name 命中)"
last_updated: 2026-10-08T04:13:25+08:00
created: 2026-10-08T04:13:25+08:00
---

自省验收 @ b27810589b26374ac32bc8cc2b17183a4a660c87(branch practice/independent-solve,HEAD ref 文件实读比对一致)。

## 项 1 发现面 - 缺(半)
- rs_search "postex dns" → 命中 postex_dns_poll(rank 2)过。
- rs_search "dns 消费件" → 4 命中(oob_serve/postex_dns/dbg_serve/postex_build),新件不在 → 缺;重复一次同结果。
- 定性:引擎为 zvec 而非 substring。证据:`filterCatalog` 是 name/description/keywords 的纯子串匹配(query 下 case-insensitive substring),而 "消费件" 这一串只出现在 postex_dns_poll 的 description 与 keywords 里;substring 路径本应只回 1 条,实际回 6 条且无一条含该串 ⇒ zg 混合腿在跑。非上限问题(zg 参数 `--limit 12`,该查仅回 4-6 条)。
- 非索引陈旧:mirror `~/.pi-rs/agent/pi-rs-script-mirror/bundled__postex-suite__postex_dns_poll.rs` 在,且 exact-name 查询能召回 ⇒ 已入索引/映射正常(mirrorName 双 `__` 不破 zgHitsToEntries 正则)。残留属 ranker 打分。与既有 dbg_cli/postex 稀释结论文档同类:search-tool.ts 里为「新 CJK 造词」加的 `--fts <q> --fuse` 词法腿,没把 description/keywords 里字面含查询词的条目抬进结果集。

## 项 2 件面 - 过
- rs_execute postex_dns_poll args=["--selftest"] → roundtrip/missing_shard_none/seq_split/terminal/b32_decode/sanitize 六项全 true,selftest "ok",exit 0。断言非桩:roundtrip 走 extract_shards→reassemble,missing_shard 走丢片重组 None,seq_split 走 parse_seq("0cc3") 读含 0x0c(c) 的歧义 hex 标签,terminal 走 parse_seq("01z2"),b32_decode 走 base32_decode("mzxw6ytboi")=="foobar",sanitize 走 sanitize_label("Marker File!")=="markerfile"。
- 顾问项(不计缺):run_task 内 `use std::io::{Read, Write};` 的 Read 未用(方法调用靠 `impl std::io::Read` 约束即可),rustc 编译告警走 stderr,不破「stdout 单信封」契约,但 `npm run check` 级洁净面会报。

## 纪律
一切经件(rs_search / rs_execute+find_files/text_grep/search_content);未改文件;未 git 提交。

