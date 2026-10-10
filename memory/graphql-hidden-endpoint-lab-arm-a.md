---
metadata:
  node_type: memory
name: "GraphQL hidden-endpoint lab arm A"
description: "lab-graphql-find-the-endpoint/A solved: /api found by {__typename} path sweep, introspection guard (textual regex) bypassed with %0A between __schema and {, carlos id=3 deleted, banner congrats"
last_updated: 2026-10-10T10:42:36+08:00
created: 2026-10-10T10:42:36+08:00
---

### 2026-02-14 lab-graphql-find-the-endpoint (arm A, solved)

Instance `https://0ae3004a034798ee80b1ee81009f0070.web-security-academy.net/`, jar /tmp/cj1.json, reused:false.

- 端点发现:url_fuzz `/FUZZ?query=%7B__typename%7D`(值表 graphql,api,api/graphql,v1/graphql,graphql/v1,api/v1/graphql,gql)→ 仅 `/api` 200(45 B,application/json),其余 404。一信封定端点,无需逐路径单发。
- 实现指纹:错误体 `Validation error (FieldUndefined@[x])` + `extensions:{}` = graphql-go(Go)。
- 拦截面:解码后 query 含 `__schema`/`__type` 即 156 B 固定错误。空格挡(`{__schema {` 仍被拦),换行放行:`{__schema%0A{...}}` 过闸 → 拦截是文本正则,`.`/`.*` 不跨行。
- 反证:`{__sch ema{...}}` 过闸但报 FieldUndefined → 拦截是文本匹配而非 AST 校验;绕过只需在 `__schema` 与 `{` 之间插换行。
- 无效绕过(实测):POST(405,Allow: GET,query 必须挂在 URL)、fat-GET body(被忽略)、重复 `query` 参数(两侧都取第一个)、`Query=` 与 `query[]=`(400 Query not present)、path 形(404)、`%73` 单字符编码(解码后命中,无效)。
- raw_matrix 编码律:请求行里字面 `{`/`}` 被服务端接受;字段分隔用逗号(graphql 忽略逗号)可免空格编码;仅换行必须 `%0A`。整条约 180 字符全百分号编码的 query 被服务端截断成 `<EOF>` 语法错,同长度字面括号形成功 → 长 query 优先字面括号。
- 模式:query `getUser(id: Int!): User{id,username}`;mutation `deleteOrganizationUser(input:{id: Int!}) { user {id username} }`;Input 类型字段只由 `inputFields` 暴露(`fields` 为 null)。
- 利用:raw_matrix `getUser(id:{{V}})` × 1-6 定 carlos=3;GET 直投 `mutation{deleteOrganizationUser(input:{id:3}){user{id,username}}}` → 回删对象;复读 `getUser(3)`=null;banner_verdict solved:true 且 congrats 行在。
