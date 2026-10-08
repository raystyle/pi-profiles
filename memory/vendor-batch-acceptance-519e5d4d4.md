---
metadata:
  node_type: memory
name: "Vendor batch acceptance 519e5d4d4"
description: "vendor 批 519e5d4d4 自省验收 3/3 过:engine=zvec + /proc exe 实测 vendor zg 路径(4 PID 对应 4 查询);发现面 postex_dns_poll/h2_burst 均 rank 1;vendor/zvec-grep+iwe 源树在仓且 target/ 被忽略"
last_updated: 2026-10-08T11:54:44+08:00
created: 2026-10-08T11:54:44+08:00
---

## 2026-10-08 — vendor 批自省验收(HEAD 519e5d4d4,核心 7ac356425)

3/3 过。权威 AGENTS.md;全程走件(sh_run/rs_search/nap),未改文件,未提交,验收后 `git status --porcelain` 仍空。

### item 1 — 二进制切换(过)
- engine=zvec:`~/.pi-rs/agent/rust-debug/debug.log` 每次 rs_search 落 `search.call {engine:"zvec"}` + `search.zg-raw {bytes,head,needsIndex}`;本轮 8 次查询全部 zvec,无 `search.zg-fail`/`zg-index-fail`。
- zg 二进制 = vendor 产物:`/proc/<pid>/exe` 扫描捕获到 4 个独立 zg PID(766561/772059/777468/783067),exe 全为 `packages/rs-agent/src/extensions/rust_script/vendor/zvec-grep/rust/target/release/zg`,时间戳与 4 次 search.call 逐一对应。
- 关键实测事实:`PI_ZG_PATH` 未设;外部克隆 `~/repos/zvec-grep/rust/target/release/zg` 也存在,故"唯一候选"论证不成立,必须靠 /proc 实测路径。resolveZg 先查 vendor 后查外部(search-tool.ts:48),实测命中 vendor。

### item 2 — 发现面回归(过)
- `rs_search "dns 消费件"` → postex_dns_poll rank 1(debug head `#1 matchedBy=fts+vector ...postex_dns_poll.rs`)。
- `rs_search "single packet race"` → h2_burst rank 1(head `#1 matchedBy=fts+vector ...h2_burst.rs:37-142`,fts+vector = 英文向量腿命中)。

### item 3 — vendor 树存在性(过)
- vendor/zvec-grep 源树在仓(400 tracked),vendor/iwe 源树在仓(467 tracked);`git ls-files -s` 无 160000 gitlink ⇒ 是仓内源树非子模块。
- `vendor/zvec-grep/rust/target` 存在(zg 78MB,Oct 8 11:20),被 `vendor/zvec-grep/rust/.gitignore:8:/target/` 忽略(`git check-ignore -v` 命中),`git status vendor/` 空 ⇒ 干净。

### 复用经验(踩坑)
- 采短命进程不能只靠 `ps -eo args | grep`;本环境 zg 活约 620ms,但忙循环的 fork 风暴会让刚 spawn 的子进程饿死导致漏采(`$SECONDS` 在 dash 也不可用)。
- 可靠手法:后台 `/proc/[0-9]*/exe` readlink 扫描,case 匹配路径子串,每轮 `sleep 0.1`,自匹配安全(exe 不会是 sh/grep);后台 run 的 stdout 到进程退出才落盘,读日志须等结束。

