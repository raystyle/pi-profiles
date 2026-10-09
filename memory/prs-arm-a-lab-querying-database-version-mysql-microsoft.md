---
metadata:
  node_type: memory
name: "PRS arm A lab-querying-database-version-mysql-microsoft"
description: "arm A 基线:lab-querying-database-version-mysql-microsoft 冷实例(reused:false)一次通过 - /filter?category= 两列 UNION SELECT @@version,NULL--+ 回读出 8.0.42-0ubuntu0.20.04.1,banner solved"
last_updated: 2026-10-09T05:21:02+08:00
created: 2026-10-09T05:21:02+08:00
---

## arm A 基线 — lab-querying-database-version-mysql-microsoft

- 实例:range_launch launch-url(canonical academy 路径)直出,reused:false;jar /tmp/cj1.json。
- 题面 hint(实例 banner 内自带,非题解页):"Make the database retrieve the string: '8.0.42-0ubuntu0.20.04.1'"。
- 注入点:/filter?category=Gifts。单引号 -> 500;两列 OK、三列 500 ⇒ 原查询两列且均为字符串。
- 解法(一条请求):`/filter?category=Gifts%27+UNION+SELECT+@@version,NULL--+`
  - 版本串在列 1;响应把 `8.0.42-0ubuntu0.20.04.1` 渲染成一行 product(th),banner 转 is-solved。
- 判定:banner_verdict -> solved:true + "Congratulations, you solved the lab!"。
- 坑:cache_probe 的 --marker 会命中实例 banner 自带的 hint 串(每行都 hit),不能用作版本回读判据;须落盘读 body 或按列位置核。
- 数量:2 次 cache_probe/1 次 http_session 探测 + 1 次落盘读 = 冷实例一次通过。

