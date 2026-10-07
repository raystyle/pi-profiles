---
title: "lab-request-smuggling-0cl-request-smuggling"
---

# lab-request-smuggling-0cl-request-smuggling

> evidences: [[h2-smuggling-family]]

PortSwigger `request-smuggling/advanced/lab-request-smuggling-0cl-request-smuggling`(批 41 新实例
`0a54001404b3a7a982386fee008e0086`)。目标:让每 5s 开首页的 Carlos 执行 `alert()`。

- 判定:**stuck**(原语在档;**没有任何可作为 JS 载荷来源的面**,投递面也仍缺)。

## 复核与新增

1. **前端同时吃 h1/h2**;`h2_req --header 'content-length: 5' --data 'ab'` 立即 200 ⇒ h2 的 CL 头被前端
   丢弃/重算 ⇒ **H2.CL 帧在本 lab 不成立**(与 [[lab-request-smuggling-h2-cl-request-smuggling]] 相反)。
2. `:path` 内 CRLF 透传成立(注入的 h1 请求会执行,可做副作用型攻击,如写评论)——批 35 已证。
3. **首页没有任何 JS 文件**(只有 `/resources/labheader/js/labHeader.js`、`/resources/css/labsBlog.css`、
   `/resources/images/blog.svg`、`/image/blog/posts/*.jpg`),且**页面里没有 exploit-link**(无 exploit server)。
   ⇒ "让 Carlos 执行 `alert()`"这条链缺**载荷来源**:lab app 没有任何把参数反射进 JS 体的端点,
   也没有可指向的外部脚本源(exploit server 不可达:走私出的请求只由 lab app 处理,见批 40)。
4. 未见跨连接后端复用(批 35 实测:完整走私后新 h1/h2 follow-up 均 200;arm 连接保持打开也一样)。

## 未决面

- 需要:①一个能产生 JS 体响应的 lab 端面(本轮未找到,且无 exploit server);或 ②受害者浏览器自己的连接
  被 CSD 式 desync(需要受害者访问我方页面——题面只说 Carlos 访问靶场首页)。

## 复现

```
lab_launch launch 4BAD74C356692BBAAB9602C955308F8A1B04CB866034F32286993566EFC8CC66 --widget-source /web-security/request-smuggling/advanced --jar <jar>
h2_req <inst> --method GET --path '/ HTTP/1.1\r\nHost: <inst>\r\n\r\nGET /404z HTTP/1.1\r\nHost: <inst>\r\n\r\n'
lab_http get <inst>/ --out /tmp/l1.html    # 首页:确认无 JS 资源
```
