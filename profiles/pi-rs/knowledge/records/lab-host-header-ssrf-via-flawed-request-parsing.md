---
title: "lab-host-header-ssrf-via-flawed-request-parsing  [stuck]"
links:
  - target: host-header-family
    relation: evidences
---

# lab-host-header-ssrf-via-flawed-request-parsing  [stuck]

> evidences: [[host-header-family]]

- 题面:routing-based SSRF via flawed request parsing;访问 `192.168.0.0/24` 内网 admin,删 carlos。
- 实例(批42R):https://0a9c00a50412498c80075850009200f2.web-security-academy.net
- 状态:**unresolved**,但本批推翻了解法成文形状(见下),并把"谁来路由/校验"钉死。

## 否证面与实测边界

1. **成文形状在本 infra 不成立**:writeup 共识=`GET https://<lab>/ HTTP/1.1` + `Host: 192.168.0.X`
   ⇒ 实测**全部** 403(`Client Error: Forbidden` 109B,digest e539ba4b26269526,**无 Set-Cookie** = 预检层失败)。
   1-20 与 1-254 两次遍历(https:// 与 http:// 请求行都试)共 274 次,零命中、零变体。
   ⇒ 该前端**始终**校验 Host 头(`Host.split(':')[0]==<lab>`),绝对请求行不构成豁免。
2. **请求行侧仍被单独校验**:`GET http://192.168.0.171/` + `Host: <lab>` → 403 但**带 `_lab`**(路由阶段),
   即预检(无 cookie)与路由(cookie 仅 `_lab`)两阶段可分,且请求行宿主不是 allowlist 站就死在路由阶段。
3. **h2 反例**:`:authority: <lab>` + 另发 `host:`(任意大小写)→ 服务端 **GOAWAY(error 0, 17B)**;
   无 `host` 头时 h2 正常 200 ⇒ 该边缘把 h2 的 Host/:authority 不一致当连接级错误(写本的 h2 形在此死)。
4. **折行(obs-fold)无效**:`Host: <lab>\r\n\tHost: <IP>` 与空格缩进两形 → 缩进行被丢弃,响应=靶场首页(10690B,
   digest 0e1a45d64882cc52)⇒ app 只读名为 `Host` 的那一条。
5. **头名变体只骗过重复检查,骗不过路由与 app**:
   `Host\t:`/`Host :` + 正常 `Host:` 组合(二进制直发,conn_reuse)可绕过 `{"error":"Duplicate header names are not allowed"}`,
   但路由与 app **都**取名为 `Host` 的那一条(先 lab 后 exploit → 落到 exploit server;先 exploit 后 lab → 路由到靶场且 src 渲染 lab)。
6. `Host: <lab>:80@192.168.0.171`、`<lab>:@192.168.0.171`、`<lab>:80@example.com` → 全部 200 靶场首页
   ⇒ 路由取 `split(':')[0]`=lab,userinfo 后缀不改上游。

## 未决面

- 要进内网需"预检读 lab、上游读 IP"的字段分裂。本 infra 上:Host 头单点、请求行单点、h2 不一致直杀、
  obs-fold 被丢、重复头名变体被同一规范读法吃掉 ⇒ 已知手段全封。仍未穷尽:请求行 authority 里 `;`/`,`/`%` 编码、
  超长/IDN 形、以及 **HTTP/1.0 与 pipelining 组合**。

## 复现

```
raw_http "<inst>/" --request-line 'GET https://<lab>/ HTTP/1.1' --header 'Host: 192.168.0.FUZZ' --ids 1-254 --quiet   # 零命中
conn_reuse "<inst>/" --send-str 'GET / HTTP/1.1\r\nHost: <lab>\r\n\tHost: 192.168.0.171\r\nConnection: close\r\n\r\n' # 靶场首页
h2_req "<inst>/" --path / --header 'Host: <lab>'      # GOAWAY
```
