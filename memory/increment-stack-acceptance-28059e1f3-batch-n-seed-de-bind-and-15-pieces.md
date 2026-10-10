---
metadata:
  node_type: memory
name: "Increment stack acceptance 28059e1f3 batch N seed de-bind and 15 pieces"
description: "验收 @28059e1f3:item1 半缺(中文串 rank 8)、item2 半缺(seed 空但项目库仍命中三本)、item3 半缺(bundled 去绑被 project tier 旧副本遮蔽)、item4 过(scan clean 89 件 exit0)"
last_updated: 2026-10-10T10:00:49+08:00
created: 2026-10-10T10:00:49+08:00
---

自省验收增量栈 @28059e1f3(批 N:seed 去题绑与 15 件晋升)。四项判定:

1. 语义召回(半缺):engine=zvec 过——/home/ray/.pi-rs/agent/rust-debug/debug.log 三条 search.call 带 "engine":"zvec"(01:58:52/54/56),zg-raw head 均 `#1 matchedBy=fts+vector`。召回:「last byte sync race burst」h2_lbs_race rank 1/12(过);「末字节同放单包竞态绕限流」rank 8/11(缺,前 7 为 upload_race/desync_probe/race_spread/qpoison/udp_send/jclass/ts_token)。⇒ 中文近义串仍不达标。

2. seed 出货面(半缺):bundled seed 目录三本答案键笔记已无(grep 空),internal-and-fragment-cache-poisoning-method.md 仍在 seed global+bundled(过)。但 `knowledge find` 对 writeup-payloads / five-stuck / key-algebra 三名仍返回命中——命中的是 .pi-rs 子模块项目库的旧副本(.pi-rs/knowledge/ 下同名 .md,mtime Oct 7 08:40)。⇒ 「前三者零命中」只在 seed 面成立,项目库面不成立。seed-exile 副本落 practice/eval/seed-exile/(commit 5088c90fd 迁出)。race-conditions-family 含写侧齐发/读侧播撒的节律句(第 11-12 行,读侧标 confirm/SELECT)——「confirm-vs-change 阵形句」在,但写侧标注是 register/INSERT 而非 change。

3. money_loop 去题绑(半缺):bundled packages/.../hunter-suite/money_loop.rs 已去绑(description「Drive a coupon-vs-redeemable arbitrage loop on a shop target...」,无题名无 SIGNUP30)。但 catalog/运行面走 project tier,.pi-rs/rust-scripts/money_loop.rs(Oct 7,子模块内)仍是旧绑:「PortSwigger "infinite money" logic-flaw loop」。不带参跑(实跑即 project 副本)退 exit 2,usage 行 `<base-url> <user> <pass> [--coupon CODE] [--target N]`(过)。⇒ 去绑未落到件面;project tier 影子遮蔽 bundled。根因:.pi-rs 是 git 子模块,5088c90fd 只改了 bundled 副本。

4. seed-binding-scan(过):practice/eval/seed-binding-scan.sh 自跑 → `binding scan clean over 89 file(s)`,EXIT=0。

纪律:全程走件,未改仓内文件,未 git 提交,未读 practice/eval 下 lab-* 运行档案。

