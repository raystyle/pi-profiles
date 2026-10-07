---
metadata:
  node_type: memory
name: "Backlog Batch Acceptance 16a952655"
description: "Re-verification of 010054a58/90405f41a: both defects closed, original pty recipe passes end to end on the documented path; final verdict 过"
last_updated: 2026-10-07T16:58:46+08:00
created: 2026-10-07T16:43:27+08:00
---

## 2026-10-07T16:45+08:00 — 联合验收单(commit 16a952655)逐项实跑

总判:**缺**(1/3/4/5 过;2 缺)。全程未提交,repo 树干净。

- **1 init 合一 过**:四项组件检查唯一实现落在 `extensions/init/component-checks.ts`(25/70/83/93 行;全 src 扫描仅此四处定义),core.ts 16-23 import、env-init.ts 16-18 import + 32-34 re-export(import 面不变,env-init.test.ts 仍从 env-init 取件)。`./pi-test.sh --no-env init --check-only` 新四项全「已达标」:bundled-rust-libs / ocr-models / zg-index / managed-tools。
- **2 pty 会话 缺**:desk 侧 pty 分支本身**可用**(协议级实证:raw proc_start pty:true + `tty` → `/dev/pts/8\r\n`;`python3 -i` → wait '>>>' 命中、proc-send 2+3 → wait '5' 命中 ">>> 2+3 5 >>>")。但两条断链:
  - **A. dbg_cli proc-start 不带旗标**:`dbg_cli.rs:156` 手拼 `json!({"cmd":"proc_start","args":{"name","command","cwd"}})` 未合并 `args`(pty 在此),其余子命令要么 clone args 要么走 `_ => {"cmd":cmd,"args":args}`。实证:`proc-start t1 --pty -- tty` → "not a tty"(exit 1)。
  - **B. dbg_serve proc_start 丢子进程 argv**:父版 `dbg_serve.rs:1067` 有 `cmd.args(&command[1..])`,重构把该行连同旧 stdio 一起删了,现文件 `.args(` 零匹配。实证:`proc-start e -- /bin/echo hello-from-echo` → 账本 text 只有 `"\n"`;`aa -- /bin/echo AA-BB-CC` → wait 未命中;py 子进程 `/proc/<pid>/fd/{0,1}` = pipe 且 cmdline 只有 `python3\0`。影响:**README/seed 里所有 gdb/strace/rr/python 配方全废**(只跑 argv[0])。
  - 对照组 pp(pipes)同样受 B 阻塞;旁证:桌外 `printf '2+3\n' | python3 -i` 无 tty 也出 `>>>`,python 自身不是 tty 判别器,判别要看 `tty` 输出或 `\r\n`(ONLCR)vs `\n`。
  - 两件 --selftest:dbg_serve 15/15、dbg_cli 4/4 全过,**但都不覆盖子进程 argv 与 proc-start 请求形状**,故两条缺陷漏网。
- **3 停起竞 过**:stop 回复 1ms 内,紧接 serve → exit 0 / running:true / 新 pid,status 立答(pid 与 lock 文件同步)。旧台 stop 后 **78ms** 内退出并摘掉自己的 sock(自定义 socket 实测)。inode 守卫**直接证明**(确定性):desk 999・其 sock 路径被换成普通文件(ino 690152→690160),靠 idle 租约退休后 **FOREIGN-SENTINEL 存活**,对照台(未换)自己的 sock 被摘 ⇒ 删换条件=仍是自己绑的 (dev,ino)。注:锁持到进程退出、摘 sock 在退出之前,故 stop→start 这条路上该窗口实际够不到,守卫属防御性,证据取 inode 置换而非竞态。
- **4 seed/README 过**:`seed/debug-desk-four-lines.md` 新节 50-57 行纯方法级(rr 会话=proc 会话 + gdb record/reverse-* 同理 + 录制区间要短;CDP 无协议级 pretty-print ⇒ 拿源离线格式化、断点回原始行),无题面键。部署副本 sha256 与仓内 seed 一致(27733032…607b7)。README:61 有 `--pty` 用法行——但该行与 seed 的 `proc-start rr -- rr replay ./bin` 今天都跑不通(受 A/B),文档描述了同 commit 打断的路径。
- **5 门禁 过**:`npm run check` exit 0(biome 1207 文件、tsc --noEmit 净);vitest env-init + init-env + rust-tool = 3 文件 **65 通过**;`rust:catalog` 重生成后 `git status --porcelain` 无输出(目录新鲜)。

环境坑(复用):desk 守护进程 comm 被截成 15 字符(`dbg_serve_bab28`),`pgrep -x dbg_serve` 永假,清残留用 `pgrep -f dbg_serve.rs`;rust-script 缓存二进制在 `/proc/<pid>/exe` 显示 `(deleted)`,核验「跑的是不是新码」要靠 `strings` 该 exe 查特征串(本轮查 `openpty` → 命中,证明新码在跑)。


## 2026-10-07

## 2026-10-07T17:05+08:00 — 复验(缺项已修,010054a58 + 90405f41a):两条缺全部闭合

- **A(--pty 不转发,90405f41a)**:`dbg_cli.rs:161` 现为 `{"name","command","cwd","pty": args.get("pty").cloned().unwrap_or(json!(false))}`,args 头列 `proc-start(--pty)`,catalog.json 同步重生成。实证(文档路径):`proc-start t3 --pty -- tty` → 账本 `/dev/pts/9\r\n`(修前 "not a tty" exit 1)。
- **B(argv 丢失,010054a58)**:`dbg_serve.rs:1101` 在两分支前恢复 `cmd.args(&command[1..]);`。实证:管道形 `proc-start av -- /bin/echo hello-argv-a hello-argv-b` → `"hello-argv-a hello-argv-b\n"`(修前仅 `"\n"`);pty 形 `proc-start pt --pty -- /bin/echo hello-pty-argv` → `"hello-pty-argv\r\n"`。
- 原验收单的 pty 配方现于**文档路径**全程走通:`proc-start py --pty -- python3 -i` → wait '>>>' 命中(banner+提示符)→ proc-send 2+3 → wait '5' 命中 `>>> 2+3 5 >>>`。
- 回归护栏:dbg_cli selftest 5/5(新增 "proc-start protocol line carries pty");dbg_serve selftest 15/15;`rust:catalog` 重生成后 git status 空。
- 教训沉淀:该 selftest 新断言是**形状断言**(对手搭字面量),不跑 `main()` 的 proc-start 臂——若该臂再次丢 pty 它仍会过;真正的守卫是端到端 echo/tty 用例。另:python3 在 tty 下无 `-i` 也进 REPL,故用 python 验 argv 会漏;`/bin/echo` 多参 + `tty` 是可靠判别子。
- 终判:**过**(1-5 全部闭合)。`.pi-rs` 子模块的 ` M` 仅为本次 memory 归档写入,非代码改动;未提交任何内容。

