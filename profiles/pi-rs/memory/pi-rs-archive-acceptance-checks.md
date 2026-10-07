---
metadata:
  node_type: memory
name: "pi-rs archive acceptance checks"
description: "How to verify the memory-de-seed / knowledge-seed-to-global batch: pty header capture, non-clobber + positive-control recipe, and two mode artifacts"
last_updated: 2026-10-06T22:22:16+08:00
created: 2026-10-06T22:22:16+08:00
---

### 记忆去 seed / 知识装 global 批验收(实测回执)

- 被验批: `917a13746 feat: memory drops the seed concept; knowledge seed installs to global`,验收期间并发会话又落 `343056cd3 fix: review's F1 - seed deletions actually land`,HEAD 已前进;测前先 `git rev-parse --short HEAD`(本 checkout 多会话并发,工作树会移动)。
- 交付面: 26 篇顶层 seed 笔记字节级等于 bundled `extensions/knowledge/seed/*.md` 落入 `~/.pi-rs/agent/knowledge/`(子目录 `tools/`、`crates/` 不装,故 26 而非 58)。
- 头部(Memory/Knowledge 各两行 project/global)只在交互 TUI 渲染,不在 print 模式 → 抓法: `script -qec 'timeout 12 ./pi-test.sh --verbose' /tmp/pi-tty.txt` 后 `text_grep "project: " /tmp/pi-tty.txt`;这次 pty 运行同时触发 session_start(=> ensureSeedInstalled),一石二鸟。
- 非破坏验证三件套: ①改 global 里 `timeout-standard.md` 头部一行 → 再跑产品 → `diff -u seed/x global/x` 只多该行;②正面控制: `mv` 走 `hunter-oracle-engineering.md` → 再跑 → 文件回来且 `diff -q` 与 seed 完全相同(证明安装确实执行);③对照 `ls *.md | wc -l` 26→25→26。
- 观察(非缺陷):
  - `knowledge find --dir <global 绝对路径>` 的 lint 报 51 条 broken-edge(同目录 fan-out lint 报 0):显式目录模式只在库内解析边,而 global 只装顶层笔记,边指向 project records 与子目录笔记。
  - `packages/rs-agent/src/extensions/memory/lint.ts:75` 残留 `if (tier.source === "seed") continue;` —— de-seed 后 getMemoryTiers 只产 project/global,该分支不可达。

