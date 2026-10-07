---
metadata:
  node_type: memory
name: "tunnel_variant_scan acceptance regression bd422fd78 recheck"
description: "tunnel_variant_scan 修订复验 @bd422fd78(实跑树 50af4b4fe):rst 断言过;发现面「tunnelling oracle」一查 rank 4 未进前 3(另一查 rank 1),判缺"
last_updated: 2026-10-07T18:32:20+08:00
created: 2026-10-07T18:32:20+08:00
---

## 复验对象与树

- 派单指定 HEAD `bd422fd78`(fix: rst_code pure fn with selftest guard);实测 `git rev-parse --short=9 HEAD` = `50af4b4fe`(docs + gitlink 两提交在其之上),`git merge-base --is-ancestor bd422fd78 HEAD` 成立 ⇒ 代码路径与被验收提交一致,复验有效。
- 纪律:全程走件(search_content/read/sh_run/rs_search/rs_execute),未改文件,未提交。

## 项 1 发现面(rs_search 信封合同=有序文本行)——判 缺(半过)

- 信封实构:`formatCatalogEntry` 每条返回两行(件名行 + `  args:` 行),条目按序拼接为 text;排名即条目出现序。
- 「漏洞猎手套件 隧道 变体」→ tunnel_variant_scan 第 1 条(两次实跑稳定)⇒ 过。
- 「tunnelling oracle」→ 顺序 blind_oracle / smuggle_seq / conn_reuse / **tunnel_variant_scan** ⇒ rank 4,未进前 3(两次实跑一致)。
- 定性:非信封合同违约(合同=有序行、无 score,已符合),是混合查询下语义排序偏置 —— 「oracle」腿拉 blind_oracle,向量腿把 HTTP/1.1 走私件排在 H2 件前。单用本件核心词排名正常:「tunnelling」→ rank 1;「tunnel variant scan h2 accounting」→ rank 1。
- 故若把判据放宽为「进入结果集(11 命中内)」则过,按派单字面前 3 则缺;建议派单或修排序期望二者取一。

## 项 2 rst 断言 —— 过

- `rs_execute tunnel_variant_scan --selftest`(3.1s):`data.rst_code == true` 且 `data.selftest == "ok"`,其余十项自测布尔(accounting/clean/converge/filler/inner_build/no_pad_honest/pad_len/pad_shape/status_read/x_cache)全 true。

## 可复用判据

- rs_search 验收计数口径:一条目 = 两文本行;「前 N 行」应按条目序计,派单措辞宜写「前 N 条」。
- 语义腿活性的判别式:结果里出现不含查询字面的件(crate_suite/google_search 等)即 zg 语义腿在跑,不是 substring 回退。

