---
title: "academy-edge-lab-cookie-gate"
---

# academy-edge-lab-cookie-gate

# academy-edge-lab-cookie-gate

Academy 实例域名前面有一层**边缘**(不是 lab app 自己),它对"头形态"利用做严格校验。裸件(不带任何 cookie)扫这类利用时会系统性拿 4xx,容易被误判成"本 infra 不支持该形状"——批42R 就是这样把两道 host-header 题判成否证的。

## 现象(两道 host-header 题上一致)

| 请求 | 结果 |
| --- | --- |
| `Host: <内网 IP>`(任意请求行) | `403 Client Error: Forbidden`(109B,digest `e539ba4b26269526`,**不下发 `_lab`**) |
| HTTP/1.0 不带 Host / `Host :` / `Host\t:` / 绝对请求行指内网 | 同样 403(边缘不回落 SNI,也不认畸形头名) |
| 两个同名 `Host:` | `400 {"error":"Duplicate header names are not allowed"}`(同样是边缘) |
| **任意上述形状 + 合法 `_lab` 实例 cookie** | **放行**,交给 lab app 继续处理 |

## 钥匙与判据

- 钥匙 = `http_session get "https://<实例>/"` 拿到的 **`_lab` cookie**(会话 cookie 不是必须的;`_lab` 单独就够)。
- 判据:**带 `_lab` 下发的 4xx 是 lab app 发的,不带的 4xx 是边缘发的**。例如 SSRF 题里 `GET https://<lab>/` + `Host: <IP>` 无 cookie → 403 无 `_lab`(边缘);`GET https://<IP>/` + `Host: <lab>` → 403 **带 `_lab`**(lab 前端的请求行 allowlist)。
- 因此:**任何 host-header / 重复头 / 缓存键分裂类假设,探针必须带 `_lab`**;无 cookie 的扫(even 274 次)证明不了任何否证。

## 解锁的两个形状

1. **routing-based SSRF**:`GET https://<实例>/<path> HTTP/1.1` + `Host: <内网 IP>` + `_lab` → 路由到内网(实测 `504 connecting to 192.168.0.171`,命中 `192.168.0.170` 拿到 admin 面板)。
2. **缓存键 vs 渲染值分裂**:`Host: <实例>` + `Host: <exploit>` + `_lab` → 缓存按第一个 Host 取键、app 按第二个渲染(`src="//<exploit>/resources/js/tracking.js"`),致毒体落进实例键。

## 关系

- 家族:[[host-header-family]]、[[cache-poisoning-family]]、[[portswigger-platform-specifics]];实例开题见 [[lab-launch]]。
