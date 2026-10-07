---
metadata:
  node_type: memory
name: "tunnel_variant_scan acceptance regression 7681b92ea"
description: "tunnel_variant_scan 自省验收 @7681b92ea(HEAD): 3/5 过; item1 rs_search 信封无 score/file, item2 selftest 无 rst 断言"
last_updated: 2026-10-07T18:08:55+08:00
created: 2026-10-07T18:08:55+08:00
---

# tunnel_variant_scan acceptance regression (HEAD 7681b92ea)

Batch: 新件落地自省验收。逐项实测 3 过 / 2 部分缺。件在册 (`scripts/catalog.json:1065`, file `hunter-suite/tunnel_variant_scan.rs`), README 表已列。

## 逐项
1. **发现面 — 缺(证据面)**: zvec 语义腿生效(query「隧道 变体 记账」整串在 substring 下不可能命中却回 8 条 ⇒ 走 zvec, 非 filterCatalog)。rank: 该关键词排 1/8; 「漏洞猎手套件 tunnelling」排 3/8(dbg_serve、h2_req 在前)。但 `search-tool.ts::execute` 只回 `formatCatalogEntry` 文本行(name/version/source/description/args) + `details:{matches,engine}` —— **信封里既无 score 也无 file**; zg compact 的 `#N matchedBy=` 排名在 `zgHitsToEntries` 已丢弃。要求里的 score/file 不可满足。
2. **selftest — 缺(rst 一面)**: 单 AgentResult 信封, `data.selftest=="ok"`, 断言字段 accounting/clean/converge/filler/inner_build/no_pad_honest/pad_len/pad_shape/status_read/x_cache 全 true。**无任何 rst 相关断言字段** —— `read_response` 的 rst_stream 解析(HEAD review round 新增)无 selftest 覆盖;全仓 rst_stream 只出现在 h2_req/h2_burst/tunnel_variant_scan 三件, 无测试文件断言它。
3. **件真跑 — 过**: `/tmp/tvs-accept.tsv` 一行 `acc<TAB>clean<TAB>0<TAB>-<TAB>-`, url `https://www.rust-lang.org` → rows[0] 含 `outer_status:"301"`、`pad_applied:false`、`rst_stream:null`(字段齐, 无注入故 null)。success=true, 单信封。
4. **失败路径 — 过**: `https://github.com/raystyle/pi-rs --converge 1`(不给 --expected)→ `success:false`, error `"no M: the clean cell carried no content-length (chunked response?) and --expected is unset"`, exit 2。chunked 源即 h2 回执无 content-length 的源。
5. **泛化 — 过**: `search_content "burpcollaborator|portswigger.net/[0-9a-f-]{8,}|X-SSL"` 该件 → files_with_hits 0。靶场地址/题面常量未入随包面。

## 纪律/方法备注
- 全程走件: rs_search / search_content / text_grep / tunnel_variant_scan / sh_run; 唯一写入是 /tmp 验收 tsv; 未 git 提交。
- 外部 https 只打 README 已列的源(rust-lang.org、github.com/raystyle/pi-rs), 单次 GET, 不扫不爆。
- 复现短语: `tunnel_variant_scan <url> --variants /tmp/tvs-accept.tsv`(件形状) 与 `tunnel_variant_scan <chunked-url> --converge 1`(失败路径)。

