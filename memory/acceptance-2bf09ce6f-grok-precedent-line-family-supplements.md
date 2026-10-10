---
metadata:
  node_type: memory
name: "Acceptance 2bf09ce6f grok precedent line family supplements"
description: "验收 @2bf09ce6f:item1 缺(三族补充只在 seed/global,project tier 旧副本经 get 遮蔽;ssrf/oauth/host-header 新行与 parser 差异表在位不了)、item2 过(oob 件召回;h2_lbs_race 召回 #6,中文 keywords 在位且 project 副本已同步)"
last_updated: 2026-10-10T10:40:29+08:00
created: 2026-10-10T10:40:29+08:00
---

补充注召回自省(批 N:grok 先例线三族补充 @2bf09ce6f)。两项判定:

1. 图关联补充(缺):2bf09ce6f 只改 bundled seed 三本(ssrf-family +10、oauth-family +7、host-header-family +8/-1),global 镜像 Oct 10 10:38 同步到位;per-tier 计数证明——seed=global 在(oauth PKCE/动态注册=2、ssrf 开放重定向/二段=2、host-header 悬断=1、路由与缓存消费者=1),`.pi-rs/knowledge/`(project tier,Oct 7 旧副本)mtime 旧、全部为 0。`knowledge get` 按 project>global>bundled 解析 ⇒ 实取项目旧副本:ssrf 矩阵只有黑名单/白名单两行、无 parser 差异表(Orange Tsai 谱);oauth 只有代理页偷令牌一行,无 redirect_uri 四档/code 拦截/state-PKCE/implicit 未绑/动态注册五行;host-header 无「路由与缓存消费者(grok 先例线)」节,无「Host 被边缘钉死时改信 X-Forwarded-Host/Forwarded…悬断标记」重置链消费者句。⇒ 与 money_loop、三本答案键笔记同根因:.pi-rs 项目 tier 的 Oct-7 快照遮蔽了 seed/global。已修不了(本轮不改文件)。

2. rs_search 语义召回(过):「带外交互回调证据信道」→ burp_collab(#1)/dns_oob(#2)/dbg_cli(#3)/oob_serve(#4)/oob_poll(#5)/postex_relay/postex_dns/postex_dns_poll,OOB 相关件召回成立;「单包竞态末字节」→ h2_lbs_race 召回(#6/12,前五 pcreg_race/race_spread/upload_race/desync_probe/qpoison);h2_lbs_race 中文 keywords 在位且 project 副本与 hunter-suite 同版(.pi-rs/rust-scripts/h2_lbs_race.rs:6 = `keywords: race, single-packet, last-byte-sync, h2, http2, concurrency, bypass rate limit, 并发, 竞态, 单包, 末字节, 同放, 绕限流`;两处均 21440B Oct 10 10:13)——这件同步到了 project tier,money_loop 那件没同步。

共性缺陷(跨两轮):凡只改 bundled seed/hunter-suite 的批,`.pi-rs` 子模块内 project tier 的旧副本都会在真实读取面(catalog、knowledge get/find)遮蔽新内容;验收必须打件面而非源文件。

