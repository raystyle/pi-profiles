---
metadata:
  node_type: memory
name: "GraphQL increment batch self-inspection"
description: "graphql 增量批自省验收:项1 过(族注两子型+CSRF 句在位)、项3 过(语义查询入前三 rank3)、项2 缺(gql_alias_brute selftest 断言 `p\\\"w1` 未算 JSON 二次转义,实测 body 为 `p\\\\\\\"w1`,exit 1)"
last_updated: 2026-10-10T11:52:08+08:00
created: 2026-10-10T11:52:08+08:00
---

验收对象:graphql 增量批(graphql-family 新注 + gql_alias_brute selftest),逐项实测。

- 项 1 过(knowledge):`get_knowledge graphql-family`(不 expand)命中,正文含「端点发现两子型」小节——默认面 `/resources/js/gqlUtil.js` 读 sendQuery 取真端点(实测 `/graphql/v1`)+ 隐藏面字典扫 `?query={__typename}` 探活并 `%0A` 绕内省正则;另含「CSRF 面」小节:x-www-form-urlencoded 的 mutation 原样被接受(sink 不看 Content-Type)⇒ 跨站表单 changeEmail 即 CSRF。两处要求在位。
- 项 2 缺(件面):`rs_execute gql_alias_brute --selftest` 回 `{"success":false,"error":"selftest failed","next_suggestion":"inspect alias builder"}`,进程 exit 1。根因(已证):selftest 的第四个断言 `body.contains("p\\\"w1")`(期望单层转义)为假——请求体是 `json!({"query": ...})` 再序列化,密码里的 `"` 与 `\` 被二次转义,body 实际携带 `p\\\"w1`(三个反斜杠);`a0:login`/`a1:login`/`mutation{`/`:login(input:` 计数四个断言均为真。POC 复现:同形 body 中 `p\"w1` 子串 False、`p\\\"w1` 子串 True。修法与其余四题同形:断言改期望二次转义形。
  证据行:`packages/rs-agent/src/extensions/rust_script/scripts/hunter-suite/gql_alias_brute.rs:26-28`。
- 项 3 过(发现面):`rs_search` 语义查询「GraphQL 别名爆破绕限速」,`gql_alias_brute` 列第三(timing_enum、acct_lock_enum 之前/之后:序列 = timing_enum → acct_lock_enum → gql_alias_brute),入前三。

旁证:无全局层遮蔽副本(`~/.pi-rs/agent/rust-scripts` 无 gql_alias_brute),执行的即 bundled 1.0.0。

纪律:全程走件;未 git 提交;未读任何 practice/eval 下 lab-* 运行档案。

