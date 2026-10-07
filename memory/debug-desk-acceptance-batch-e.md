---
metadata:
  node_type: memory
name: "Debug Desk Acceptance Batch E"
description: "复验 1-b @7591c2ebe:仍缺,流量拦截调试 = dbg_serve#1/dns_oob#2/tunnel#3/dbg_cli#4,catalog 已 regen 含字面词组仍不改序 ⇒ 残留属 ranker 打分"
last_updated: 2026-10-07T19:45:02+08:00
created: 2026-10-07T19:30:51+08:00
---

自省验收回归 调试台批 E(net scan 流量拦截扫描),树 HEAD ab5d90011(`feat: debug desk batch E - net scan, breakpoint-captured active audit`)。逐项实测:1 缺(半)+2/3/4/5 过。

- **item 1 发现面 = 缺(半过)**。`rs_search 流量拦截扫描` 有序前 3 = dbg_serve / tunnel_variant_scan / page_snapshot,**dbg_cli 在第 5**;换查询 `流量拦截` 同样 dbg_serve #1、dbg_cli #5。`rs_search 活扫 audit` = dbg_serve / h2_req / dbg_cli → 该项过。两件 .rs(`debug-suite/dbg_{cli,serve}.rs` 第 6 行 keywords)都带「流量拦截扫描」,但 bundled `scripts/catalog.json` 的 keywords 数组止于旧集:dbg_cli 13 条(缺 流量拦截扫描/拦截/traffic),dbg_serve 16 条(有 活扫/scanner/audit,缺同样后三条)。生成器 `scripts/gen-rust-catalog.mjs:51` 只 split/trim,无条数上限 ⇒ 是「加词后未重跑 `npm --prefix packages/rs-agent run rust:catalog`」的漂移,不是生成器截断。注意:ranker 仍能命中只存在于 .rs 的词 ⇒ 搜索面读取的是脚本 `//!` 元数据,stale catalog 只影响排序权重。
- **item 2 selftest = 过**。`dbg_cli --selftest` → data.selftest ok(5 checks);`dbg_serve --selftest` → selftest ok,**24 checks**(scan 纯函数面 9 条在内:slots 四型/json_set/差分三档/battery 安全约束/build 两式)。
- **item 3 活测 = 过**。`dbg_serve --socket /tmp/accept-e.sock`(pid 1306112,ledger /tmp/accept-e-ledger.jsonl)→ `dbg_cli attach` 直接命中已开 tab(engine-profile 里预存的 `https://portswigger.net/web-security?query=dast`,冷启才会 spawn)→ 后台 `net scan --pattern '*' --battery xss --max-probes 4 --wait-ms 30000` → nap 3s → `dbg_cli send Page.navigate --params '{"url":"https://portswigger.net/web-security?q=x"}'` 驱动同 tab → 回执:baseline.status=200(数字)、baseline.len=31444、probes=4、template.slots 16 条非空(`query:q` + 14 个 cookie + `header:User-Agent`)、suspects=[](JSON 数组)、skipped_cookie_unsafe=2。`dbg_cli stop` → stopped true / engine_killed false;随后 status 报 socket No such file ⇒ 收尾干净(浏览器留存)。前置:模板源必须**从台所在主机直连可达**(重放腿是台内 h1 客户端,不经浏览器),先 `http_dump` 验一次 200 再开扫。
- **item 4 账本 = 过**。/tmp/accept-e-ledger.jsonl 内 4 条 `method=scan.probe`,params 均带 `slot`(query:q 与 cookie:*)`payload`(`zxqj<svg onload=alert(1)>` / `zxqj'><script>alert(1)</script>`)`family`(xss)`placement`(replace)`diff`(hit=false,len_delta=0,status=200,reflected=false,signature=[]);另有 1 条 `scan.report`(与回执同形),无 `scan.releaseFailed`。
- **item 5 泛化 = 过**。`search_content "burpcollaborator|oastify|portswigger.net/[0-9a-f-]{8,}"` 扫 debug-suite 目录 = files_with_hits 0 / matched_lines 0。

纪律:全程走件(dbg_serve/dbg_cli/http_dump/nap/search_content/text_grep/find_files/sh_run),仅写 /tmp,未改文件未提交。

复用要点:`rs_execute` 的后台运行不能只带 attach 读回执(参数校验要求 name/attach/ps 恰一,实传 attach 仍被拒),直接读 `~/.pi-rs/agent/rust-runs/<bg-id>.log` 即可拿信封。

后续:补跑 catalog 生成把三条词写回(不改文件纪律下仅记录,未执行)。


## 2026-10-07

复验 @HEAD 8e2d058b7(`fix: batch E review round … three-pillar names`,`git rev-parse` 对齐)。只复验 item 1 发现面,改判:**query 1 过、query 2 缺**。

- `rs_search 流量拦截扫描` → 1 dbg_serve / 2 tunnel_variant_scan / 3 **dbg_cli**(上轮为 #5)⇒ 两件进前 3,**过**。
- `rs_search 流量拦截调试` → 1 dbg_serve / 2 dns_oob / 3 tunnel_variant_scan / 4 **dbg_cli** ⇒ 缺;重复一次同序,稳定。
- 元数据面已修实:catalog.json 两件 keywords 均含 5 条新词(dbg_cli 列表 165-171 行,dbg_serve 193-199 行;版本升 1.4.1)。⇒ 上轮「加词未 regen」的根因已消除;残留是 **ranker 打分**(两件与目标词同权,却排在 dns_oob/tunnel_variant_scan 之后,疑为语义/模糊腿在 调试/扫描 上的漂移),不是关键词缺席。

另:item 2/3/4/5 本轮未复测(不在复验范围),版本已从 1.4.0 升 1.4.1。


## 2026-10-07

复验第二次 @HEAD 7591c2ebe(`fix: dbg_cli description carries the three pillar names (discovery ranking)`)。只测 1-b:

- `rs_search 流量拦截调试` → 1 dbg_serve / 2 dns_oob / 3 tunnel_variant_scan / **4 dbg_cli** ⇒ **仍缺**;与 8e2d058b7 一轮完全同序(两次运行稳定)。
- 元数据面确认已 regen 到齐:`catalog.json:151` dbg_cli description 已字面含「流量拦截调试/流量拦截扫描/JS执行断点单步多步跟踪调试三面的驱动面」,两件 keywords 亦含该词(168 行 dbg_cli、196 行 dbg_serve);版本 dbg_cli 1.4.1 / dbg_serve 1.4.2。
- ⇒ 结论:把目标词组写进描述仍未改变排序,ranker 不是「串出现即高分」的纯字面腿;dbg_serve(描述不含该三面词、仅 keywords 有)反居 #1,dns_oob/tunnel_variant_scan(两者都不含该词组)压过 dbg_cli。残留是 ranker 打分模型本身,需从其打分实现入手,不是元数据。

对照:同批 1-a `流量拦截扫描` 在 8e2d058b7 已过(#1 dbg_serve/#2 tunnel_variant_scan/#3 dbg_cli)。

