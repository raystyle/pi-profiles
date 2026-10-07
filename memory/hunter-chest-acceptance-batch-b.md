---
metadata:
  node_type: memory
name: "Hunter Chest Acceptance Batch B"
description: "Batch B acceptance (tree 376aaf5f7): 5/5 passed - dns_oob live loop recipe, cred_matrix CRLF-template requirement, blind_oracle transport bucket, h2_req arg-parse proof"
last_updated: 2026-10-07T10:38:34+08:00
created: 2026-10-07T10:38:34+08:00
---

## 猎手武器库验收批 B — tree 376aaf5f7(实测时 worktree 干净,git status --porcelain 0 行),结论 5/5 过

验收项与方法(全部经 rs_execute 走件,产物只在 /tmp):

- catalog:dns_oob 1.0.0、cred_matrix 1.0.0、blind_oracle 1.1.0、h2_req 1.2.0 均可见。
- dns_oob 实跑:PI_TIMEOUT_SECS=6 + 端口 5353 + a-answer 127.0.0.1 → 信封 `queries=1, last_qname=marker.test.oobs, elapsed_ms=6101`,exit 0;日志 /tmp/dns-oob-accept.log 落 JSONL 证据行 `{"ts":…,"qname":"marker.test.oobs","qtype":1,"src":"127.0.0.1:43007"}`。
- cred_matrix:模板 + 2 槽 → combinations:3,cred_1/2/3(mask 01/10/11);cred_3.txt 字节为 `…X-Admin: 1\r\n\r\nBODY`(双槽头行齐 + 头体间空行)。
- blind_oracle --classify 不可达目标(http://127.0.0.1:9/):success:true,buckets 一条 `transport/<url> … Connection refused (os error 111)`,distinct_states:1。
- h2_req:`--data A --pad-to 32` 对闭端口只出连接失败信封(解析+填充通过);正控 `--data AAAAAAAAAA --pad-to 4` → `--pad-to 4 <= current body 10`(exit 2,守卫在);`--hdr2-file` 指不存在路径 → `cannot read …: No such file or directory`(exit 2)。

可复用配方(客户端腿缺位时的 dns_oob 刺激法):套件内没有 UDP 发送件(dns_oob 只有 UDP/TCP 服务面,oob_poll 只 ssh 读远端日志),因此用 sh_run 落盘 34B 查询再 nc 发出:
`printf '\022\064\001\000\000\001\000\000\000\000\000\000\006marker\004test\004oobs\000\000\001\000\001' > /tmp/q.bin` + `nc -u -w1 127.0.0.1 5353 < /tmp/q.bin`;实测 attempts=1 即收回 A 应答(`12 34 85 00 … c0 0c 00 01 … 7f 00 00 01`),服务端计数 queries。这是 POC 通道(interpreter/binary 壳),若要进可复现记录须转成 rs 件。若日后要补,缺口就是"UDP DNS 客户端件"。

模板注意:cred_matrix 的槽头按 `{head}\r\n{extra}\r\n{rest}` 拼装,模板须自带 CRLF;用 LF 模板时首行保留 \n(混合行尾,仅观感,头体间仍是 \r\n\r\n)。

旁注(未修,批内不改仓):rust-materialized 编译 cred_matrix 出 unused import `std::path::Path` + unused `id`,blind_oracle 出 unused mut/`samples` 警告,不影响验收。

