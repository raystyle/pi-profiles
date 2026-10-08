---
title: "burp-collaborator-public-polling-method"
links:
  - target: oob-callback-family
    relation: evidences
  - target: blind-injection-family
    relation: evidences
  - target: request-forgery-family
    relation: evidences
---

# burp-collaborator-public-polling-method

Keywords: burp collaborator, oastify, oast, oob, polling, biid, dns exfiltration, blind ssrf, blind sqli, os command injection

**公开 Burp Collaborator 可以不装 Burp 直接轮询**——这是外传型 OOB 题的读取面。判定层的"到达即翻牌"另见 [[oob-callback-family]]。

## 前提事实

- 载荷域(`oastify.com` / `burpcollaborator.net`)与轮询域分开:**载荷域 `*.oastify.com`,轮询主机 `polling.oastify.com`**(旧客户端 `polling.burpcollaborator.net`)。
- 任取子域的 DNS 都会被应答(通配到一台 AWS NLB),但**只有"由你持有的 secret 派生出来的标签"才可轮询回来**:
  随机编造标签(如 `b53p1a2b`)能让 lab 翻检测型题,却读不回任何东西。
- **交互读出即消费**:一次 poll 拿到就没了,本地要立刻落盘。

## secret 与标签(必须逐字实现)

```
ALPHABET = "abcdefghijklmnopqrstuvwxyz0123456789"
secret   = 32 随机字节;  biid = base64(secret)
label(secret, counter):
  key_hash = first20( base36_big_endian(SHA1(secret)) )      # base36 用上面的 ALPHABET;不足 20 位左侧补 'a'
  key_hash = left10 + chk(left10) + right10 + chk(right10)   # chk(s) = ALPHABET[ Σ ord(c) % 36 ]  <-- 用 ASCII 码,不是字母表下标
  plaintext = key_hash + "1g" + hex(counter) + "y" + <可选自定义前缀> + "z"
  salt = 两个随机 ALPHABET 字符 (s1,s2);  state=[s1,s2]
  for i,ch in plaintext: reg=i%2; out=ALPHABET[(idx(ch)+idx(state[reg]))%36]; state[reg]=out
  label = s1 s2 ALPHABET[(ord(s1)+ord(s2))%36] + 密文        # 总长 30
载荷 = <数据>.<可选前缀>.<label>.oastify.com
```

## 轮询

```
GET https://polling.oastify.com/burpresults?biid=<urlencoded base64 secret>
  Accept: application/json
-> {"responses":[ {protocol, interactionString(=label), clientPart,
                   data:{subDomain | request/response(base64), type, rawRequest},
                   time(ms), client, clientPort} ]}
空 -> {}
```

`subDomain` 是完整查询名:首标签就是外传数据(如 `peter-9ebGxU.<label>.oastify.com`),HTTP 交互则在 `data.request`(base64)。

## 工具

件 **`burp_collab`**(project 层,批 53 新建):

```
burp_collab new [--custom PREFIX]     # 建/续 context,出下一条载荷(host + biid 存 ~/.pi-rs/agent/burp-collab-state.json)
burp_collab poll [--json]             # 取回交互,sub_domain/prefix/客户端 IP/原始报文
burp_collab list                      # 看已发载荷与计数
burp_collab --selftest                # 自持 secret 派生 → 解析器查一次 → 轮询取回
```

## 实测坑

- **自测的 DNS 必须走系统解析器**(`getaddrinfo`):直接向 `8.8.8.8` 发 UDP A 查询会被**本地/上游缓存应答**,
  权威根本收不到,于是 poll 永远空(批 53 首版 selftest 即踩此坑;换成 `ToSocketAddrs` 后 5/5 命中)。
- 本机 `getent hosts <随机子域>.oastify.com` 偶发瞬时 NXDOMAIN(再次查询即好),**不能**据此判断标签无效。
- 标签第一段可以有自定义前缀(`login-ssrf.<label>.oastify.com`),用于一次载荷分辨多个注入点。
- 在 HTTP 头里注入时注意 cookie 值不能含 `;`(详见 [[records/lab-out-of-band-data-exfiltration]] 的载荷选型)。

## 实测记录

- 派发算法与轮询格式对照三个独立实现核对:`ryarmst` gist(算法)、`projectdiscovery/collaborator`(`burp.go`)、
  `Groppoxx/OAST-Community`(可跑客户端);`burp_collab` 与 gist 有两处偏差被纠正:checksum 用 ASCII 码求和、base36 编码后左补 `'a'` 到 20 位。
- 件自测:5 条 DNS 交互取回全中(`cipher_reversible=true`);跨实现互证:Python 客户端轮询 `burp_collab` 生成的标签,3 条全中。
- 端到端解题见 [[records/lab-blind-out-of-band-data-exfiltration]]、[[records/lab-out-of-band-data-exfiltration]]、
  [[records/lab-shellshock-exploitation]]。
