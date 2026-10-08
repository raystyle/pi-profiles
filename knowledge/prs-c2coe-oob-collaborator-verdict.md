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

验证状态:[假设] 机制由两仓实证拼接推演,单点待实测-一发 `xxx.oastify.com`
子域查询后读 lab 状态文本翻不翻 solved。

## 边界

- 仅限 lab 判定面:学院靶场的出纤白名单是判定机制,不是通用事实。
- 真实授权目标的 OOB 仍需自有信道(pi-rs 两条腿:aws deaddrop 47.131.34.33 /
  cf_oob;见 [[oob-callback-family]])。
- 数据外传题(非纯盲检测)打到边只能过判定,若需**回读外传数据**则随机子域不够
  -子域不可控即回读面缺;此类题仍结构性受限,除非判定只要「interaction 发生」。
