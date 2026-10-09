---
metadata:
  node_type: memory
name: "PRS lab solve infoleak version control history"
description: "arm A 基线:lab-infoleak-in-version-control-history 冷实例一次通过 - .git/logs/HEAD 取两条提交,新件 git_dump 解松散对象取旧 admin.conf 口令 mhk8p9c4zrmpd3liorbd,administrator 登录后删 carlos,banner solved"
last_updated: 2026-10-09T06:54:36+08:00
created: 2026-10-09T06:54:36+08:00
---

## 2026-10-09 arm A: lab-infoleak-in-version-control-history

终态 solved(banner `is-solved` + `Congratulations, you solved the lab!`),冷实例 reused:false,实例 https://0aee00240421139280d88fbb00140056.web-security-academy.net/。

链路(range_launch launch-url 直吃 canonical 路径 → 取号一次成功)：
1. `/.git/HEAD` = `ref: refs/heads/master`,`/.git/logs/HEAD` 泄两条 reflog：初始 `9a4afac7…`「Add skeleton admin panel」+ 当前 `cf30a5a4…`「Remove admin password from config」。
2. 新件 `git_dump`（.pi-rs/rust-scripts/git_dump.rs，v1.0.0）走 HEAD/refs/packed-refs/reflog → 递归取松散对象并 zlib 解压 commit/tree/blob → 按提交列出每个文件内容。7 个对象取回，`admin.conf` 旧版泄 `ADMIN_PASSWORD=mhk8p9c4zrmpd3liorbd`。
3. http_session：GET /login 取 csrf → POST /login(administrator + 该口令) 302 `/my-account?id=administrator` → GET /admin 200 → GET `/admin/delete?username=carlos` 302 → banner_verdict solved。

件面新增能力：`git_dump <base-url> [--git .git] [--jar PATH] [--out DIR] [--match REGEX] [--max-objects N] [--snippet N] [--selftest]`，松散对象型暴露 .git 的通解（pack 型会进 missing_objects 并显式报「packed objects unsupported」）；selftest 走合成 zlib 对象的 inflate/parse round-trip（2 断言）。

