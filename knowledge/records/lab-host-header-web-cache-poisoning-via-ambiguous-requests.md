---
title: "lab-host-header-web-cache-poisoning-via-ambiguous-requests  [stuck]"
links:
  - target: host-header-family
    relation: evidences
  - target: cache-poisoning-family
    relation: evidences
---

# lab-host-header-web-cache-poisoning-via-ambiguous-requests  [stuck]

> evidences: [[host-header-family]], [[cache-poisoning-family]]

- 题面:污染首页缓存使访客执行 `alert(document.cookie)`。
- 实例(批42R):https://0a6c00d8033a310a8130f3cb001000ee.h1-web-security-academy.net(exploit `exploit-0a27002b038631f68101f200017a0049`)
- 状态:**unresolved**。本批把"缓存只坐在靶场 app 路由之内"钉死,并否证了成文的重复头形状。

## 实测机制与否证面

1. **唯一反射点**:首页仅 1 处 Host 反射 ——
   `<script src="//<Host 全值>/resources/js/tracking.js">`(grep 全页确认)。
2. **缓存只在 lab-app 路由内**:`Host: <exploit>` 或 `:authority=lab + Host: <exploit>` 的响应来自
   `Server: Academy Exploit Server`,**不带 X-Cache/Age**,也**不进**靶场键:
   - 存 `/resources/js/tracking.js` = `alert(document.cookie)` + `Cache-Control: max-age=600`;
   - 用绝对请求行指 lab(`GET http://<lab>/…`)+ `Host: <exploit>` 打一发,随后 `GET /resources/js/tracking.js` + `Host: <lab>`
     → **X-Cache: miss**,body 仍是靶场真 JS(70B)⇒ exploit server 的响应永不被缓存到 lab 键。
3. **重复 Host 被封**:`Host: <lab>` + `Host: <exploit>`(含大小写变体)→ 400 `{"error":"Duplicate header names are not allowed"}`。
   用二进制直发绕过该检查的头名变体(`Host\t:` / `Host :`,conn_reuse)`:`Host\t/expose-<lab>`+`Host:<exploit>` 与反向都试,
   **路由与 app 都只认名为 `Host` 的那条**(前者落 exploit server,后者落 lab 且 src 渲染 lab)⇒ 拿不到"缓存键 lab / 渲染 exploit"的分裂。
4. **obs-fold 无效**:`Host: <lab>\r\n\tHost: <exploit>` → 缩进行被丢,src 渲染 lab。
5. **h2 大写 Host 覆盖成立但不致毒**:`:authority:<lab>` + 大写 `Host:<exploit>` → 200 且 body 是我们的 JS
   (证明该边缘按覆盖后的 Host 路由),但随后以 HTTP/1.1 `Host:<lab>` 读同一路径仍是 miss;h2 单独 `:authority:<lab>` → 400 `{"error":"Invalid request"}`。
6. **端口原语复核(已知)**:`Host: <lab>:7777` → X-Cache miss 且 body 带 `//<lab>:7777/...`;`Host: <lab>` 读回 X-Cache hit。
   app 侧 Host 规则 = `hostname[:纯数字 0..65535]` 末端锚定;`:abc`/`:`/`:80x`/`:80.`/`:99999`/`:80@exploit` 全 500 `No host found`。
   浏览器唯一换主机手段是 userinfo(`//<lab>:80@<exploit>/…`),恰被该规则挡掉。

## 未决面

- 可投毒内容只能是 **lab app 渲染的响应**,而 app 渲染值 = 名为 `Host` 的头(必须 `<lab>[:digits]`)
  ⇒ 只能改端口,不能改 src 主机;exploit server 的响应永不入 lab 缓存。缺口仍须找"缓存键=lab 而 app 读到 exploit"的第三种头形态
  (未穷尽:HTTP/1.0+pipelining、`Transfer-Encoding` 包裹、h2 下 `:authority` 与 `Host` 的第三种拼法、`%` 编码头名)。

## 复现

```
conn_reuse "<inst>/" --send-str 'GET /resources/js/tracking.js HTTP/1.1\r\nHost\t: <lab>\r\nHost: <exploit>\r\nConnection: close\r\n\r\n'   # 只认名为 Host 的那条
raw_http  "<inst>/" --request-line 'GET http://<lab>/ HTTP/1.1' --header 'Host: <exploit>'                                  # 路由到 exploit,不入靶场缓存
raw_http  "<inst>/" --request-line 'GET /resources/js/tracking.js HTTP/1.1' --header 'Host: <lab>'                          # X-Cache: miss(读回未致毒)
```
