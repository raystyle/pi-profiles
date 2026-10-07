---
metadata:
  node_type: memory
name: "Debug Desk Acceptance Batch B"
description: "Batch B (31dda619c) proc-session acceptance: batch content 过 - gdb chain/race/strace/proc-kill all evidenced; idle lease polls every 30s so the \"7s empty retire\" expectation missed (measured 21.6s); acceptance pinned to the commit because the tree was mid-batch-C"
last_updated: 2026-10-07T13:35:36+08:00
created: 2026-10-07T13:35:36+08:00
---

Commit 31dda619c (debug-suite batch B: proc sessions - gdb/strace form) accepted 2026-10-07. Verdict: batch B content = 过 (6/7 items clean, item 6 timing expectation missed with measured numbers); the *working tree* was red during the whole run because another session's batch C WIP is landing live.

## How the acceptance was pinned (repeat this when the tree is mid-flight)

The working tree was dirty and MUTATING while I worked: `scripts/debug-suite/dbg_serve.rs` (+476) and `dbg_cli.rs` (+108) uncommitted batch C edits (Breakpoint struct, Debugger.paused handling, Fetch.continueRequest / net intercept). The bundled `dbg_serve` does not even compile in that state - `rs_execute name=dbg_serve --selftest` dies with E0382 (borrow of moved value `params`, materialized dbg_serve.rs:518, `params` moved into `call_on` then `params.take()`).

Workaround that keeps rs_execute's injection/lint and does not shadow the real names for other sessions:
1. `mkdir -p ~/.pi-rs/agent/rust-scripts && git show <commit>:<piece path> > ~/.pi-rs/agent/rust-scripts/<name>_b2.rs`
2. edit only the `//! name:` line (dbg_serve_b2 / dbg_cli_b2) so the global tier does not shadow `dbg_serve`/`dbg_cli` for concurrent sessions
3. drive everything with those names; delete the staged files when done.
For test-suite verdicts at the commit: `git worktree add --detach /tmp/<x>/wt <commit>` + `ln -s <repo>/node_modules wt/node_modules`, run vitest there, then `git worktree remove --force` (never touches the main worktree's files).

Also: the default socket `~/.pi-rs/agent/debug-desk/desk.sock` was held by another session's desk (flock rejected me with `another desk holds ... pid=699633`). Private sockets (`--socket /tmp/<x>/desk.sock`, tmux -L form) kept both desks independent; never send stop to a desk you did not start.

## Evidence per item

1. dbg_serve 15/15 ok (paths, socket siblings, flock, ledger roundtrip, until->paused, substring fallback, reply envelope, ws bare/trailing-slash port clue, /devtools direct, relative socket absolutized, proc prompt across chunk boundaries sid-filtered, snippet window, since cursor, attach opts default/explicit); dbg_cli 4/4.
2. Full gdb chain on a rustc-compiled symbolized binary worked: prompt wait -> `b main` -> "Breakpoint 1 at 0x153b0" -> `run` -> "Breakpoint 1, 0x00005555555693b0 in main ()" -> `bt` -> print -> `set confirm off` -> `quit` -> proc-list `alive:false exit_code:0`.
   Rust gotchas (record for later desks): `b main` lands on the compiler-generated C main shim which has NO line table - `print x` answers "No symbol 'x' in current context" and `next` reports "Single stepping until exit from function main, which has no line number information". Break by file:line or path instead (`break demo.rs:2` -> "Breakpoint 2, demo::compute (a=41, b=1) at /tmp/dbgB/demo.rs:2", `print a` -> `$2 = 41`, `bt` -> "#0 demo::compute ... #1 ... in demo::main () at /tmp/dbgB/demo.rs:8").
   `wait --until-output` scans from the session start by default (since 0), so prompts that arrived before the wait still match; `--since N` opts into new-output-only. gdb's stderr lands as its own stream (`stream:"err"`), e.g. "No symbol 'x' in current context".
3. Discipline confirmed: gating each command on the prompt made all 8 chain commands land. Race reproduced with a single burst proc-send whose line carries an embedded newline ("quit\nprint 7*6", one write = two queued lines): gdb printed "Quit anyway? (y or n) [answered Y; input not from terminal]" and exited, so the raced second line was DISCARDED - the expected `$N = 42` never appears anywhere in the session (`wait --until-output '= 42'` -> timed_out). Nuance: with stdin as a pipe, gdb auto-answers its inserted prompts ("[answered N; input not from terminal]" for the debuginfod ask on a stripped binary), so on this desk the loss mode is "raced command never executes" (and can kill the session early) rather than "consumed as the answer".
4. strace observer: `proc-start st -- strace -e trace=openat echo x` -> wait openat hit ("openat(AT_FDCWD, \"/etc/ld.so.cache\", O_RDONLY|O_CLOEXEC) = 3"), `read --sid st` returned 6 chunks, session exited naturally with code 0.
5. `proc-kill` on `sleep 3000`: reply killed:true; proc-list `alive:false, exit_code:null`; ledger `{"method":"proc.exit","params":{"code":null,"sid":"sl"},"seq":107}`; OS-level `ps -p <pid>` finds nothing. Actual recorded code is null (Rust ExitStatus::code() is None for signal death), not -9.
6. Lease: live procs pin the desk (`idle_secs_left: null` = u64::MAX; status answered at uptime 34s with --idle-secs 5, i.e. it survived the lease tick that fired 30s in). Retirement happens only on the lease poll tick, and that tick is a fixed 30s `tokio::time::interval` - measured: kill at start+38.4s, `desk.idleLeaseExpired` written at start+60.0s = 21.6s after the kill. So the acceptance sheet's "proc-kill 后再等 7 秒台空退" is not achievable in general; retired latency is bounded by the tick (0..30s+idle_secs). Deviation, not a crash.
7. Docs/catalog: catalog.json carries dbg_cli 1.1.0 (proc-start/send/list/kill + -- <command...> + --sid/--until-output) and dbg_serve 1.1.0; README piece table and usage block carry the proc face. `npm run check` on the real tree EXIT=0 (biome applied no fixes). vitest rust-tool: 30/30 in a detached worktree of 31dda619c; 29/30 in the dirty tree, the single failure being "catalog.json matches the scripts directory" - attributed with a direct buildCatalog comparison: committed tree MATCH, working tree DIFF at the dbg_cli description line (batch C renamed it, catalog.json not regenerated).

## Command trivia

`rustc -O0` is not a flag ("Unrecognized option: '0'") - use `-C opt-level=0`. `pkill -f <pattern>` self-matches the `sh -c` command line (kills its own shell; sh_run reports exit code null and later commands never run) - use the bracket trick `pkill -f 'dbg-accep[t]/profile'`, same for `pgrep -af 'gd[b] --quiet'`. `command -v a b c` under dash printed only the first hit; query tools one at a time.

