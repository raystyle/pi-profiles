---
title: client-side-desync-family
---
# client-side-desync-family

浏览器侧失步(CSD):让**受害者浏览器自己**的连接与服务器失步。与 server-side smuggling 的区别:
不需要前端/后端两段式,单服务器也可;前提是目标**不支持 HTTP/2**(浏览器优先 h2,故 lab 用 `h1-` 域名)。
件:`conn_reuse`(找向量)、`browser_suite`/exploit server(投递)、`lab_http`(读回泄漏)。

## 攻击形态

1. **向量**:`POST /` 这类"服务器级重定向/静态面"回 302/200 但**不读 body** → body 留在 socket 上。
   探测:`conn_reuse <inst>/ --send-str 'POST / HTTP/1.1\r\nHost: <H>\r\nContent-Length: 93\r\n\r\nGET /404probe HTTP/1.1\r\nHost: <H>\r\n\r\n' --send-str 'GET /en?cb=1 HTTP/1.1\r\nHost: <H>\r\n\r\n'`
   → 第二条拿到 `404`,向量成立。
2. **投递(fetch)**:首请求用 `mode:'cors'`(302 无 CORS 头 → 浏览器报错**不跟随重定向**),在 `.catch()` 里
   发 follow-up(`mode:'no-cors', credentials:'include'`),后者与前缀拼成走私请求。
3. **前缀即"一条完整请求头 + body 开头"**:走私请求的 `Cookie` 只能用前缀里写的(浏览器给 follow-up 加的
   `Cookie` 落在**body**里) → 前缀里带**攻击者自己的 session**,csrf 才过;受害者的 Cookie 反而被当作文本存下来。

## 泄漏 gadget 与 CL 窗口(关键坑)

- gadget = 应用里"能把 body 存成公开可读文本"的面(本 lab: `POST /en/post/comment` → 评论公开)。
- 前缀: `POST /en/post/comment` + `Cookie: session=<ATTACKER>` + `Content-Length: N` + body
  `csrf=<ATTACKER TOKEN>&postId=1&name=…&email=…&website=…&comment=`
- follow-up 的整段字节(request line + 头 + **Cookie** + 空行 + body)成为评论正文 → 泄漏受害者的 Cookie 头。
- **N 必须 ≤ 实际可用字节**(否则应用等 body、永不入库)。浏览器把 `Cookie:` 放在头块末尾,故:
  - 把 follow-up 做成 **POST + 几 KB 的 padding body**,让"头块末尾 → 总字节"之间有大余量;
  - `N ∈ [P_body + 头块长度, P_body + 头块 + padding]` 都能成立(实测 CL=1200 + 3000 padding 稳)。
  - 只发一次 follow-up 可能因浏览器头差异失败 → `setTimeout` 200ms/1500ms 各重试一次。

## 复用手法速查

- 目标必须是 h1 站点;`fetch` 要 `credentials:'include'`(否则请求不进"带 cookie 的连接池")。
- 成功判据不是页面状态,而是**应用里出现了含受害者 Cookie 的新文本**;exploit `/log` 里 `(Victim) … Chrome/154` 是投递证据。
- 投递链:`POST exploit/ formAction=DELIVER_TO_VICTIM`(必须 `--follow`,并**整份重发** responseHead/responseBody,
  每次 POST 都会覆盖存储)。

## 关联

- HTTP/1.1 基础组见 [[request-smuggling-family]];H2 系见 [[h2-smuggling-family]]。
- 方法论:web-vuln-methods(seed 层,按名引用)。
