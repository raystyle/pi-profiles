---
metadata:
  node_type: memory
name: "expert-subagent-proposal feasibility self-inspection"
description: "专家 subagent 提案内功面 5 项验收:①过(数组注册位,division 段硬编码需核心小改)②过(仓内 examples/extensions/subagent 已 spawn pi --mode json -p + batch-desk.sh 实跑形)③风险(bg 管理器私有于 rust_script,不在扩展 API 面)④过(--append-system-prompt/before_agent_start 两条注入,PI_RS_MEMORY_INDEX 靠 env 继承,只关读不关写)⑤风险(子 session-dir 可表达但无先例,listSessionsFromDir 不递归故父目录看不到子档)"
last_updated: 2026-10-10T12:13:13+08:00
created: 2026-10-10T12:13:13+08:00
---

2026-10-10 自省验收:practice/eval/expert-subagent-proposal.md(专家 subagent 并行面)产品内功可行面 5 项。全程走件(read/search_content/sh_run 探 --help),未 git 提交,未读 lab-* 档案。

判型:①过 ②过(有仓内先例)③风险 ④过 ⑤风险。

① 扩展注册位可达 —— `packages/rs-agent/src/extensions/index.ts` 的 `builtInExtensions` 字面量数组(7 项,`subagents` 已占一位)+ `builtin-factories.ts` 的 `subagentBuiltinFactories`(子会话工厂集,memory 刻意缺席)。`InlineExtension` = 工厂函数或 {name,factory,hidden?,replaceable?,builtin?}(core/extensions/types.ts:1856)。
风险:提示词 `built_in_division` 段是 core/system-prompt.ts:132 的硬编码散文 ⇒ 「Script 面加 spawn 工具行」= 核心小改,插件不能自足;工具本身只要 `definition.promptSnippet` 就在 tools 段自动列出(agent-session.ts:3488-3491)。新名 `subagent` 与既有 `subagents` 并存 ⇒ 工具名撞车靠 replaceable 让位,docs/extensions.md 的「seven built-in extensions」计数需同步。

② 子进程起法 + 信封回传 现成 —— `packages/rs-agent/examples/extensions/subagent/index.ts` 就是本方案原型:`spawn(process.execPath 或 "pi", ["--mode","json","-p","--no-session"])`,逐行 JSON.parse 事件流,`getFinalOutput(messages)` 取最后一条 assistant 文本,另带 `--model/--thinking/--tools/--append-system-prompt <临时文件>`。`practice/eval/batch-desk.sh:47` 实跑形:`env PI_RS_MEMORY_INDEX=off timeout N ./pi-test.sh -p --session-dir '$D' --session-id 'eval-<lab>-<arm>'`。shipped `./pi-test.sh --help` 实测含 --print/-p、--mode、--session-dir、--session-id、--tools、--append-system-prompt、--no-session。
缺口:提议的「子终态一行 AgentResult 回父」文本约定无先例,先例是 json 事件流里取 assistant text;text 模式 stdout 是否纯 assistant 文本未实测(【待确认】)。

③ bg 租约/kill 不能直接复用 —— rs_execute 面的 background/attach/ps/stop 确实存在(background.ts:27-211,状态机 running/exited/timed_out/killed/spawn_failed、per-run log、attach 只回增量、stop 杀进程、timeout 杀树),但管理器 `RustBackgroundRuns` 是 rust_script 私有(context.ts 的 RustToolContext.background,`start:(scriptPath,args,cwd,env)=>operations.start(...)`),core/extensions/types.ts 全文检索 background 零命中 ⇒ 不在扩展 API 面上,只能照形自建或先抽公共模块(它 deps 已注入,泛化成本不高)。另一条可复用线是 subagents 自身(agent-manager + get_subagent_result/steer_subagent,Agent 默认后台+完成通知),但它管进程内会话,不是 OS 子进程。

④ role 注入点齐 + 开关自动继承 —— 进程形:`--append-system-prompt`(args.ts:122,help 明示 text or file contents,示例写临时文件投递);进程内形两条:`before_agent_start` 返回 systemPrompt(memory/index.ts 同款)与 runner 的 `forceSystemPrompt`(runner.ts:1421-1432);另有 custom agent .md 的 systemPrompt frontmatter(subagents/custom-agents.ts)。`PI_RS_MEMORY_INDEX=off` 仅被 memory 扩展 before_agent_start 读一次(memory/index.ts:132,命中即整块 return:索引+rules+handoff 全免),spawn 不带 env 时子进程按 Node 默认继承 process.env ⇒ 继承成立,评测侧正在用(batch-desk.sh:47 前缀)。
风险:开关只关读、不关 write_memory 写面;它不是 CLI 旗标,子命令行无法再加固,只能靠继承。

⑤ 子 session-dir 挂父目录:机械可行但无先例且列档不递归 —— SessionManager.create/open 收任意 dir 并 mkdirSync recursive(session-manager.ts:1011/1755-1786),`--session-dir '<parent>/sub-<role>-<n>'` 可直接表达;但仓内两种既有约定都不是这样:进程内子会话落父同一个 default/configured sessionDir + 仅 parentSession 元数据(agent-runner.ts:947-970,/resume 里嵌套),子 transcript 落 tmpdir(`join(tmpdir(),"pi-rs-subagents-<uid>",encoded,sessionId,"tasks")`,output-file.ts:53-69);且 `listSessionsFromDir` 不递归(session-manager.ts:941-971 只读直接 *.jsonl)⇒ 子目录挂进父目录后父目录列表看不到子会话,「臂档案完整性」需另设读取路径或改平铺命名。

落点结论:②④两项有现成件面,①是数组加一项 + 一处核心提示词小改,③④⑤是自建/约定风险,不需先动 ADR 之外的结构。
