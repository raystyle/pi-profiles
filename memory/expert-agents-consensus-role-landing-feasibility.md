---
metadata:
  node_type: memory
name: "Expert agents consensus role-landing feasibility"
description: "专家 agent 清单 A–H 的 role 落地自省票:tools 白名单可窄到单工具(实测 harness 拒文)、frontmatter 默认 replace(ADR 要的追加须显式 prompt_mode: append)、严格 YAML 的 \": \" 陷阱致静默回落 general-purpose、子档 tmpdir + 子 session 与父平级落盘"
last_updated: 2026-10-10T12:29:30+08:00
created: 2026-10-10T12:29:30+08:00
---

自省票(四台一致轮,候选 A–H 的 role 落地可行项)。全程走件;未 git 提交;临时探针角色文件已删(.pi-rs/agents 现为空)。

**① tools 白名单形(可行,且可窄到单工具)**
- frontmatter `tools:` 对自定义角色全效,与 bundled 先例同一加载路径(bundledAgentsDir 与 projectDir 走同一个 loadFromDir);先例原文 `tools: read, ext:rust_script/rs_search, ext:rust_script/rs_execute, ext:knowledge`(agents/web-vuln-hunter.md、agents/deep-research.md)。
- built-in 面只有 read/edit/write 三个;扩展工具必须 `ext:` 前缀,裸写会被当未知 built-in(agent-runner 的 knownBuiltins 检查只出 tools-error 活动事件)。
- **窄化按工具**:parseExtSelectors 以首个 `/` 切分,`ext:rust_script/rs_search` 只放行该工具;同 ext 的多个 `/tool` 选择器取并集。实测(临时角色 `tools: read, ext:rust_script/rs_search`):子工具面 = read + rs_search;子主动试调 rs_execute 被 harness 拒,原文 `No such tool: rs_execute. Available: read, rs_search.`。⇒ rs_search(只读发现)与 rs_execute(执行)完全可分。
- 只读 = `tools: read`;执行 = `tools: none, ext:rust_script/rs_execute` 或 read+execute。
- 陷阱:白名单粒度是工具级,**不是脚本级** —— 不能只放行 banner_verdict 之类单个 rs 脚本。
- 「单发」不是硬约束:`max_turns: 1`(0=无限)是软上限,硬停 = max_turns + graceTurns(默认 5);graceTurns 只从 subagents.json/设置面读(最小 1),frontmatter 无字段;且 agent 自身的 max_turns 优先于派单参数。

**② 正文注入链(默认 replace,ADR 要的追加须显式写)**
- `.md` body → `config.systemPrompt`(trim);`promptMode = fm.prompt_mode === "append" ? "append" : "replace"` ⇒ **默认 replace**:short replaceHeader + `<active_agent name>` + env + body,父身份与 harness 纪律都不继承;runner 另用 systemPromptOverride/appendSystemPromptOverride 抑制 AGENTS.md/CLAUDE.md/APPEND_SYSTEM.md。
- ADR-0016 决定条 4 写的「`--append-system-prompt` 追加形,不替换」在进程内 Agent 面上的对应物是 `prompt_mode: append`(父提示 + sub_agent_context bridge + `<agent_instructions>body</agent_instructions>`)。角色文件不显式写就落 replace,与 ADR 相悖。
- 路由面 = `description` frontmatter,渲染进 Agent 工具描述 `- <name>: <desc> (Tools: <formatToolsSuffix>)`;存在截到首句的紧凑渲染 ⇒ 路由句放首句。
- **frontmatter 是严格 YAML**(utils/frontmatter.ts 用 `yaml` 包的 parse):未加引号的值里出现 ": " 直接抛 `Nested mappings are not allowed in compact mappings`;loader 只 console.warn 后跳过该文件,派单**静默回落 general-purpose(全工具面)**。A/B 实测:description 带 "tools: " 一次 → 回落(且子拿到了 read/edit/write/rs_execute 全档);仅把该值加引号 → 角色如期解析(显示名 Selfcheck Tools Probe)。
- 空/全空白 `name:` 会退回文件名;含 `:` 的 name 被拒不加载。

**③ S1 实测面(子档 tmpdir + 父目录可见性)**
- 子 transcript:`/tmp/pi-rs-subagents-<uid>/<encoded-cwd>/<父 sessionId>/tasks/<agentId>.output`(JSONL,isSidechain),按父 session 分桶,不在项目树内;上一轮 S1 探针档同形(…/s1-probe/tasks/*.output)。
- 子 session 档:与父**同目录的平级兄弟文件**(`~/.pi-rs/agent/sessions/--mnt-wsl-repos-pi-rs--/<ts>_<uuid>.jsonl`),listSessionsFromDir 不递归也可见 ⇒ ADR 的「平铺」天然成立;但文件名是生成式,frontmatter 只有 session_dir/persist_session,`NewSessionOptions.id`(core/session-manager.ts:53)没被接线 ⇒ ADR 的 `sub-<role>-<n>` 命名要一处小改。
- 父目录可见性:子 cwd = 父 cwd(实测回 /mnt/wsl/repos/pi-rs)⇒ 项目级 rust-scripts / .pi-rs/knowledge 都可解析;但 subagentBuiltinFactories 刻意不含 memory ⇒ 角色正文不得指示 write_memory;knowledge 在集内,要图必须显式 `ext:knowledge`(回落子自报有 write_memory,判【待确认】,疑为 append 提示回声)。
- 探针配方(可复用):写临时 `.pi-rs/agents/_x.md`(description 必须引号)→ Agent(subagent_type=_x)→ 收子自报工具面 + 试调被禁工具读 harness 拒文 → 删文件。

**候选面归属**:A/C/D/E/F/H = `read + ext:rust_script/rs_search + ext:rust_script/rs_execute(+ext:knowledge)`;B(verdict-verifier)是唯一声明面与工具面冲突的:自行取 banner 只能走 rs_execute ⇒ 要么真只读(父在派单里给 banner 文本),要么承认是执行面复核位 + max_turns:1;G 裁定不增,无落地面。H 另需件契约(//! 头/report 信封/catalog 重生成)在 AGENTS.md(子会话被抑制)⇒ 需 `prompt_mode: append` 或把契约抄进 body。

