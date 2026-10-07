---
metadata:
  node_type: memory
name: "rs_search Semantic Fix Acceptance"
description: "rs_search semantic fix (bf15ef74a) acceptance: 5/5 pass in a fresh process; this session's process predates the fix so its rs_search still returns substring/0; cold-start index refresh verified; no test guards the anchor mapping"
last_updated: 2026-10-07T11:15:54+08:00
created: 2026-10-07T11:15:54+08:00
---

## rs_search 语义搜索修复验收(tree bf15ef74a,结论 5/5 过)

**头号发现:验收必须在修复提交之后启动的进程里做。** 本会话进程 465803 起于 10:36:21,修复提交 bf15ef74a 在 11:10:54 → 会话内的 rs_search 仍是修复前代码(无 search.zg-raw 日志、旧锚点 `(?:^|[/\\])` 匹配不到 mid-line 名字 → 每条语义结果静默落回子串):实测 'race condition burst concurrent' 返回 "No scripts match"(engine substring,matches 0)。改仓库 TS 源码后旧会话不会热加载,会话内命令无法自救;须重启会话。

修复前/后代码靠 debug.log 判定:修复前无 search.zg-raw 行,修复后有(带 needsIndex),search.call 的 engine 因此从 substring 变 zvec。

新进程实测(冷启动,先 rm -rf ~/.pi-rs/agent/pi-rs-script-mirror/.zvec-grep):
- 首个查询 needsIndex:true(1961ms,显式 zg --index 刷新)→ engine zvec,7 命中,#1 h2_burst(race_email/race_send/h2cl_seq/h2_req/blind_oracle/conn_reuse);后续 needsIndex:false ~430ms;重建后索引 7.3M。
- 'ocr scanned pdf' → zvec/9,前三位 doc_read、doc_ocr、doc_find。
- 'ssrf portscan intranet' → zvec/12:conn_reuse、http_session、jwt、h2_req、range_launch、h2cl_seq、doc_find、text_sub、doc_ocr、raw_matrix、crate_suite、dns_oob(SSRF 家族件 header_scan/blind_oracle/url_fuzz/objref_scan 未进前 12)。
- '武器库' → zvec/3,全是猎手套件:dns_oob、fingerprint、jclass。

子串基线(filterCatalog 是**整串字面**匹配,不分词):多词查询一律 0 命中 → 语义路是 0 vs N 的差别;单词 CJK '武器库' 子串 2 命中(cred_matrix、dns_oob),与 zvec 的 3 命中互不包含(带字面关键字的 cred_matrix 被语义排名漏掉)。

/init 检查单行:checkZgIndex() → {id:"zg-index", label:"zg catalog index", ok:true, detail:"/home/ray/.pi-rs/agent/pi-rs-script-mirror"};该行只挂在交互检查单(interactive-mode.ts:6403),**init 工具的行表里没有**。debug.log 最新 search.call 的 engine 全为 zvec。

复用配方(新进程验证工具代码,不改仓):临时 mjs 里 `import {createRustSearchToolDefinition} from "<repo>/packages/rs-agent/src/extensions/rust_script/search-tool.ts"`,再 `tool.execute("verify",{query},undefined,undefined,{cwd:repo,isProjectTrusted:()=>true})`;`cd <repo> && node --experimental-strip-types <file>`(node v24.20.0 直接吃 .ts,无需构建)。这样走的就是 rs_search 自己的执行路径,search.call/search.zg-raw 由产品代码写出;属探针,不进可复现记录。

缺口:packages/rs-agent/test/rust-tool.test.ts 只覆盖 mirrorCatalog(拷贝/清理)与"zg 停用回落子串",**没有断言锚点→zvec 的映射**,而原 bug 正是"每条语义结果静默落回子串" —— CI 拦不住这个回归。

