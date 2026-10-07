---
metadata:
  node_type: memory
name: "rs_search mirror-header fix acceptance 6b105f78d"
description: "发现面修复批 6b105f78d 自省验收:3/3 过 - 消费件/dns 消费件 召回 postex_dns_poll 均 rank 1;后渗透套件 通道 前 8 中 postex 6 件;同查询连发排名一致且 needsIndex=false/mirror+index mtime 未动"
last_updated: 2026-10-08T04:45:09+08:00
created: 2026-10-08T04:45:09+08:00
---

## 2026-10-08 自省验收 @6b105f78d(rs_search 镜像头挪位)

终判 3/3 过,零缺。

1. 发现面(此前 0 召回):过。`rs_search "dns 消费件"` → postex_dns_poll rank 1(head 命中 `bundled__postex-suite__postex_dns_poll.rs:15-126`);`rs_search "消费件"` 亦 rank 1。反证:`消费件` 在镜像里只出现在 line 6 `//! description` 与 line 9 `//! keywords`(search_content 确认),代码体内无此词 ⇒ 召回只能来自头文本入索引。
2. 广度:`rs_search "后渗透套件 通道"` 前 8 = postex_dns / postex_dns_poll / postex_exec / postex_relay / postex_build / fingerprint / postex_http / desync_probe ⇒ postex 6 件(此前 1/5)。跨族抽查:`指纹引擎 规则表` → fingerprint rank 1(头独有词组)。
3. 幂等稳定:同一查询 `消费件` 连发两次排名逐位一致(8 命中);debug log 两次 `search.call` engine=zvec,`search.zg-raw` needsIndex=false,ms 623 / 618。四问全部 needsIndex=false。镜像目录 mtime 04:37:14、manifest 04:38:21 在我的四次查询后均未变,件数 96。单测独立复跑 `vitest --run test/rust-tool.test.ts` = 31/31 过。

### 复用配方(下次验收发现面)

- **先判会话是否带修复**:pi-rs 由 `./pi-test.sh` 源起进程,扩展模块进程启动时装载。`ps -o pid=,ppid=,lstart=,cmd= -p <pi-rs pid>` 对比 `git show -s --format=%ci <commit>`;`ps` 祖先链从 `$$` 上溯即可定位本会话 pi-rs 进程。
- **did-it-reindex 的可观测面**:`~/.pi-rs/agent/rust-debug/debug.log`,kind `search.zg-raw`(字段 bytes/head/needsIndex)与 `search.call`(matches/engine/ms)。needsIndex=true 约 1.5-1.7s,settled 约 0.6s。
- **镜像实地路径**:`~/.pi-rs/agent/pi-rs-script-mirror`(平铺 `<source>__<file>` + `.zvec-grep/` 索引),96 件;`stat` 其 mtime + manifest mtime 即可证"本轮未重写"。
- 本会话首跑未触发重写:镜像在修复作者 04:37 的活验证轮已 settle,故 needsIndex 从第一问起即为 false(属预期收束态,不是缺测)。

