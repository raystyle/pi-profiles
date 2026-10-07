---
title: "lab-perform-web-cache-deception"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-perform-web-cache-deception

> evidences: [[request-smuggling-family]]

- 题面:Exploiting HTTP request smuggling to perform web cache deception;前端不支持 chunked、前端缓存静态资源;
  目标 = 走私让"下一个用户的请求"把其 API key 存进缓存,再取出提交(`POST /submitSolution`,`answer=`);无 exploit server。
- 实例(批40):https://0ae100d5047aaa2980c73a20005c0007.web-security-academy.net;`wiener:peter` 可登录,`/my-account` 回显 32 位 API key。
- 状态:**unresolved**(机制全貌已明:缓存判定=Cache-Control、路由按 Host、走私请求不可再路由;缺"后端连接复用"这一步)。

## 已确认的面

1. **缓存判定 = `Cache-Control: max-age`**,不是"路径静态":
   - `/resources/js/foo.js`(404,app 仍给 max-age:30)→ 第二次 `X-Cache: hit` ✓
   - `/nonexistent.txt`(404,无 cache 头)→ 不缓存(无 X-Cache)✓
   - `/post?postId=1`(200,无 cache 头)→ 不缓存 ✓
   ⇒ app 对 `/resources/js/*` 一律加 `max-age:30`(含 404)⇒ 任何落在该前缀下的响应都可缓存。
2. **前端按 Host 路由,且直达任意 host**:`GET /resources/js/tracking.js` + `Host: <exploit-server>` → 得到 exploit server 的
   `"Resource not found - Academy Exploit Server"` 404(可据此区分 app 的 JSON `"Not Found"`)。
3. **走私出来的请求不会被二次路由**:它在"前端↔靶场后端"这条连接里,只能由靶场 app 处理;exploit server 访问日志里
   只有我自己的 ureq 请求,没有任何走私请求 ✓(批34 把外层 POST 的响应误认成走私响应)。
4. 缓存键**含 Host**:用 `Host: <exploit>` 拿到的响应不会命中靶场 Host 的同一个 URL(`X-Cache: miss`)。
5. 前端对我方连接发 `Connection: close`;同一连接里再发第二个请求拿不到它的响应(无法在同一连接内做 response-queue 错位)。

## 正确的攻击形状(未收口)

前缀法:走私请求的**头块不闭合**(如 `GET /my-account HTTP/1.1\r\nX-Pad: `),让受害者的请求字节补进来 ——
受害者的请求行成为 `X-Pad:` 的值,其 `Host:`/`Cookie:` 变成走私请求的头 ⇒ 后端按**受害者会话**返回 `/my-account`(含 API key),
而前端把这条响应记在**受害者那个静态子资源请求**的 URL 上 ⇒ 缓存里出现 API key ⇒ 取回提交。
卡点:需要前端的"前端↔后端"连接池被受害者连接复用(批34/批40 均未观察到复用)。

## 复现

```
lab_launch launch 2BEAC929D831E232F42BA851C86EF44AB27D908ABCC5D78ED79A5CBC6C710A13 --widget-source /web-security/request-smuggling/exploiting --jar <jar>
conn_reuse <lab>/ --path / --cl-te 'GET /my-account HTTP/1.1\r\nX-Pad: ' --read-ms 3000   # 前缀法 arm
http_dump <lab>/resources/js/tracking.js                                                 # 查是否被投毒
```
