---
metadata:
  node_type: memory
name: "PRS arm A lab-determine-number-of-columns"
description: "arm A 基线:lab-determine-number-of-columns 冷实例(reused:false)一次通过 - union select NULL 三列全绿,banner is-solved"
last_updated: 2026-10-09T04:53:45+08:00
created: 2026-10-09T04:53:45+08:00
---

arm A 基线(A 臂 = 优化树),目标 lab: /web-security/sql-injection/union-attacks/lab-determine-number-of-columns。

## 实例与入口
- page_read canonical 路径直接给出 widget-lab-id(C0CF223D…B517B307),无 404。
- range_launch launch-url(带 --jar /tmp/cj1.json)→ 冷实例 reused:false,
  https://0aa800f303b9c03282c23ed30088008f.web-security-academy.net/。
- 首页过滤链:/filter?category=Corporate+gifts(注入点 = category)。

## 解法(一次通过)
- raw_matrix 一条 §UNION SELECT NULL…-- 扫描 n=1..5 × 两种注释(--+ 与 --+-),共 10 变体。
- 结果:n=3 两式均 200(n3-a 5170B / n3-b 8194B,各带新 session cookie);
  n=1/2 与 n=4/5 全 500(SQL 列数不匹配)。⇒ 查询返回 3 列。
- banner_verdict:/tmp/cj1.json 首页 is-solved=true,congrats 行出现。判 solved。

## 坑
- raw_matrix 自带 Host;变体里再写 "Host: …" 会触发
  {"error":"Duplicate header names are not allowed"} + 400(全变体同 50B 同 digest)。
  规则:变体 headers 只放额外头,Host 交给 spec.host。
- raw_matrix 无 cookie jar,本 lab 不需要(匿名 filter 即可)。

