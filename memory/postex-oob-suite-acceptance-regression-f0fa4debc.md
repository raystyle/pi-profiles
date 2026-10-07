---
metadata:
  node_type: memory
name: "Postex + oob-suite Acceptance Regression f0fa4debc"
description: "后渗透套件批 1 + oob-suite 批回归验收 @f0fa4debc(item1 半缺:rs_search CJK 查询丢 postex_exec;item3 过但需手动 chmod;item4 缺:oob_serve 无 usage/信封)"
last_updated: 2026-10-08T01:19:17+08:00
created: 2026-10-08T01:19:17+08:00
---

## 2026-10-08T01:20:00+08:00 回归验收 @f0fa4debc(postex 套件批 1 + oob-suite 批)

树:HEAD f0fa4debc29a4a32313efc132fc26d45d9630b8a(前一笔 b2fc8a0fa = postex 套件批 1)。
纪律:全程走件,未改仓,未提交。逐项裁决 过/缺 + 证据。

### 逐项
1. 发现面 —— **半缺**。
   - `rs_search 「postex playbook」`:postex_build #1、postex_exec #2 → 过。
   - `rs_search 「后渗透套件」`:postex_build #3、**postex_exec 整列未现**(两次询问同序 ⇒ 非抖动)。
   - 判别式:`rs_search 「后渗透套件本地传输件」`(exec 自己描述的字面前缀)exec 仍缺席、build #1;
     而 `rs_search 「postex_exec」` exec #1 ⇒ **索引里有它,是打分/截断问题,不是索引缺口**。
     同列还挤进描述既无「OOB/后渗透」也无「套件」的 dbg_serve/race_spread/tunnel_variant_scan/fingerprint。
     —— 与 debug 批 E 的 ranker 残留同族(该批 regen catalog 字面词未改序)。

2. selftest —— **过**。
   - postex_build:7 具名布尔全 true + `selftest:ok`(scaffold_toml/scaffold_wrap/content_hash/arch_parse/
     sdk_embedded/raw_delimiter_guard/braces_passthrough)。postex_exec:2 真 + `selftest:ok`(frame_roundtrip/eof_none)。
   - 计数注记:题述「九断言」只有按两件合并计(7+2=9)才对得上;单看 build 是 **7 非 9**。

3. 活测全链 —— **过(带一个未文档化前置)**。
   - `/tmp/ph-pb.rs`(file::Write /tmp/ph-postex-marker + shell::Run id,照 README 形)。
   - build:444280B 静态 musl,built_ms 2388,hash_head f9733634e0 → ~/.pi-rs/agent/postex-bin/ph-pb-x86_64-f9733634e0。
   - **首跑 postex_exec 直接失败:`Permission denied (os error 13)`** —— postex_build 用 `fs::write` 落 0644,
     postex-suite 源码 0 处 chmod/PermissionsExt ⇒ 手工 `chmod +x` 才通(README 未提这一步;
     bin 目录里批作者的 pb-marker 两件为 +x,亦属手工)。
   - 首跑 `ok:true, steps:2, changed:2`,hello{arch:x86_64,os:linux,proto:1},events=[marker/changed, id/changed];
     **进度行 `[changed] marker …` 走 stderr、信封在 stdout(两流分立实测)**。
   - 重跑 `ok:true, steps:2, changed:1`(marker→ok/clean,id→changed)⇒ 幂等成立。marker 正文 "ph\n" 3B,已删。
   - 附带:失败路径回执合同正确(cargo exit 101 → `success:false` + 原始 cargo 输出 + CTA,exit 1);
     但**失败路径漏 scaffold 临时目录**(/tmp/postex-build-<pid>-<ms>,实测新起一个 5→4 归来)。
     仓内现有 4 个(01:13:43–01:14:21,含 Cargo.lock)即批作者迭代期失败编译残留,成功路径会清。

4. oob-suite 面 —— **半缺**。
   - `rs_search 「OOB套件 canary」`:oob_poll #1、dbg_serve #2、**oob_serve #3**(5 条内第 3)→ 进前列,勉强过;
     但 dbg_serve(描述无 OOB/canary)压其上,排序质量注记。
   - `--help`/缺参回执 —— **不存在**:oob_serve.rs 内 `help|usage|selftest` 0 命中、`report::success|failure` 0 命中
     ⇒ **任何路径都不产 AgentResult 信封**;main() 位置参数解析失败即静默回落(port→53)并直接 bind 起服,
     唯一早退是 bind 失败(裸 eprintln + exit(1),非信封)。harness 面亦不拦 --help(executor 无拦截)
     ⇒ 实跑必然起服务,按纪律不跑,以源码判缺。信封合同违例(AGENTS.md:legacy 违例是缺陷不是先例)。

5. init 面 —— **过**。`checkZigBuild` 定义 component-checks.ts:118、导入 core.ts:23、调用 core.ts:813;
   spec 表 core.ts:810-811 `id:"zigbuild" / name:"cargo-zigbuild 静态导出链"`(component-checks 另有三处 zigbuild 行)。

### 三处待修(按影响排)
1. postex_build 产物不置 +x ⇒ 文档化链 `build → postex_exec <binary>` 首次必然 EACCES(一行修)。
2. oob_serve 无 usage/--help 且无信封 ⇒ 任何调用都起服务,合同与可用性双缺(应给 --help/selftest 早退分支 + report 信封)。
3. postex_build 失败路径漏 scaffold 目录(成功路径已清)。

