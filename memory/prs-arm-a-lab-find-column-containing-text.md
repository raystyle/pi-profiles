---
metadata:
  node_type: memory
name: "PRS arm A lab-find-column-containing-text"
description: "arm A 基线:lab-find-column-containing-text 冷实例一次通过 - /filter?category= 三列,第 2 列字符串列"
last_updated: 2026-10-09T05:00:01+08:00
created: 2026-10-09T05:00:01+08:00
---

### 2026-10-09 arm A 基线实录(lab-find-column-containing-text)

- 实例:range_launch launch-url /web-security/sql-injection/union-attacks/lab-find-column-containing-text --jar /tmp/cj1.json → reused:false,instance_url 直接可用(18.2s)。
- 题面取值:实例首页 banner 自带 `<p id="hint">Make the database retrieve the string: 'zY62NB'</p>`(无须 page_read 题解页)。
- 注入点:`/filter?category=Gifts` 的 category 串;占位约定 `' UNION SELECT ...--`(PostgreSQL,`--` 不带尾空格被接受)。
- 列数=3,字符串列=第 2 列:`' UNION SELECT NULL,'zY62NB',NULL--` 返回 200,响应表格里直接渲染出 `<th>zY62NB</th>` 行;第 1 列(%27zY62NB%27,NULL,NULL)与第 3 列(NULL,NULL,%27zY62NB%27)均 500 Internal Server Error ⇒ 列位置判定即那一发。
- 判定链条:第 2 列请求本身 200 且含目标串;其对偶两发 500 作反证;第三次请求的 banner 已翻 `is-solved` + `Congratulations, you solved the lab!`;banner_verdict(jar 同一 API 实例的会话)复验 solved:true。
- 有效路径:只要一发命中字符串列即 solved(该 lab 判定即"查询返回了指定串",不需后续数据提取腿)。
- 并发注意:三发探针同一 block 并行发出、不带 --jar(匿名浏览不需要会话),避免同一 jar 并发写;但并发导致第 3 发的响应里已含 solved banner —— 反证用的 500 仍成立,解读时勿把"并发交错"误判为注入失败。
- 过程特性:纯 HTTP,4 次 rs_execute(range_launch + 三发探针) + 1 次 banner_verdict 收口;零浏览器、零 OOB。

