---
metadata:
  node_type: memory
name: "Debug Desk Acceptance Batch A"
description: "Batch A (45f293bc1) debug-suite acceptance: 7/7 过 - PPID=1 desk, isolated-chrome attach, console wait hit, timed_out lease, flock double-desk + SIGKILL revive, docs/catalog in sync; recipes for isolated test chrome + pkill self-match trap"
last_updated: 2026-10-07T12:46:49+08:00
created: 2026-10-07T12:46:49+08:00
---

Commit 45f293bc1 (debug-suite batch A: 持久任务调试台 dbg_serve + dbg_cli) accepted 2026-10-07, 7/7 过, no gaps.

## What was proven (envelope evidence)

1. `dbg_serve --selftest` 7/7 ok (paths, socket siblings, flock semantics, ledger roundtrip, until->Debugger.paused, subscribe substring fallback, reply envelope); `dbg_cli --selftest` 4/4 ok.
2. `dbg_serve --idle-secs 600` -> running:true pid=659016; `dbg_cli status` -> pid/uptime_s/attached:false; `ps -o pid,ppid,pgid,sess` -> PPID=1, PGID=SESS=self (setsid escape works).
3. attach into an isolated test chrome with bare `--ws ws://127.0.0.1:9333` -> attached:true, spawned:false, target = the test page. The port-clue normalization (bare ws://host:port -> /json/version discovery) is the working path; the shared `~/.pi-rs/agent/chrome/engine-profile` was never spawned or touched.
4. `wait --until console --timeout-ms 8000` -> matched[0].method=Runtime.consoleAPICalled (waited_ms 14). `read --since 0` -> count 8 spanning Debugger.scriptParsed / Runtime.consoleAPICalled / Page.frameStoppedLoading. Ledger `~/.pi-rs/agent/debug-desk/desk-ledger.jsonl` lines carry seq/ts/method/session_id/params.
5. `wait --until paused --timeout-ms 1500` -> success:true + timed_out:true + waited_ms 1501: lease timeout is a normal result, not an error.
6. Double desk rejected: `another desk holds .../desk.lock: pid=659016`. `stop` -> stopped:true, daemon pid gone, desk.sock unlinked, engine-profile chrome (12 procs) untouched = detach semantics. Restart after graceful stop -> new pid. `kill -9` left stale socket + lock content 661650, `dbg_cli status` -> Connection refused; a fresh `dbg_serve` then succeeded (flock self-release + stale socket removed/rebound, pid 662026). Final stop + test-chrome teardown left no desk, no socket, no /tmp profile.
7. AGENTS.md line 8 = "Seven suites: ... debug-suite ..." and line 10 extension list includes debug-suite; system-prompt.ts line 136 carries the 持久任务调试台 sentence (dbg_serve daemon outlives every client, dbg_cli drives attach/read/wait/stop); catalog.json lines 150-191 hold dbg_cli/dbg_serve with file paths; `vitest --run test/rust-tool.test.ts` 30/30 (catalog freshness guarded by buildCatalog); debug-suite/README.md has the same shape as browser-suite/README.md (title, prose, 件 table, rs_execute usage lines) plus extra 进程治理/工程注 sections.

## Reusable recipes

Isolated chrome for desk/attach acceptance (never the shared engine-profile):
`setsid ~/.browse-rs/chromium/<pinned>/chrome --user-data-dir=/tmp/dbg-accept/profile --remote-debugging-port=9333 --no-first-run --no-default-browser-check --no-sandbox --headless --noerrdialogs --ozone-platform=headless --ozone-override-screen-size=800,600 --use-angle=swiftshader-webgl file:///tmp/dbg-accept/page.html > chrome.log 2>&1 < /dev/null &`
Page = console.log on a 1s interval + location.reload every 4s, so console and navigation events keep arriving on their own.

Pitfall (one wasted round): `pkill -f 'user-data-dir=/tmp/x/profile'` self-matches the `sh -c` command line and kills its own shell (sh_run exits with code null, later commands in the one-liner never run). Use the bracket trick - `pkill -f 'dbg-accep[t]/profile'`. Same trap when grepping for a process: `pgrep -af 'dbg_ser[v]e'`.

Ledger rotation keeps exactly one previous generation (`desk-ledger.1.jsonl`): each desk start removes the old .1 and renames the current ledger onto it, so a start followed by no attach leaves a 0-byte .1 - normal, not a defect.

