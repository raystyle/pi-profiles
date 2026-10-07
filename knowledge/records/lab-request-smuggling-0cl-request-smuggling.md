---
title: lab-request-smuggling-0cl-request-smuggling
---

# lab-request-smuggling-0cl-request-smuggling

> evidences: [[h2-smuggling-family]]

PortSwigger `request-smuggling/advanced/lab-request-smuggling-0cl-request-smuggling`。批 44 实例 `0a5c00fe03e83bd1805b17ef00ce0051`(批 41 的 `0a520096…` 在批 44 中途已 504 `connecting to <inst>` 死掉)。目标:让每 5s 开首页的 Carlos 执行 `alert()`。

- 判定:**stuck**(机制面进一步收窄;仍缺可交付的载荷来源与 0.CL 帧。)

## 批 44 新增证据

1. **畸形头 `Content-Length : N`(冒号前空格)在本实例不产生 0.CL 差分**:一次 write 发
   `POST /resources/css/anything`(带 `Content-Length : 92`)+ 紧跟 `GET /404probe …` ⇒ **同连接拿到两条响应**
   (`302 Location: /resources/css/anything/` + `404 "Not Found"`)⇒ 前后端对帧长的读法一致(无人 holding)⇒ 该畸变头不是这里的 0.CL 原语。
   - 附带确证:**early-response gadget 存在**——静态目录路径 `/resources/css/anything` 立即回 302(不等 body),正是 0.CL 死锁所需的解结面。
2. **h2 侧不成立**:`content-length: 100` 无 DATA ⇒ 立即 200(前端丢弃/重算 CL)⇒ 无 H2.CL/0.CL 帧。
3. `:path` 内 CRLF 仍透传(注入的 h1 请求会执行),可作为**副作用**面(写评论等),但不构成 alert 交付。
4. 首页资源清单:`/resources/labheader/js/labHeader.js`、`/resources/css/labsBlog.css`、`/resources/images/blog.svg`、`/image/blog/posts/*.jpg`;**无 exploit server**(`#exploit-link` 不存在),无自产 JS 体。
5. 唯一 XSS gadget:`GET /post?postId=N` 把 **User-Agent 原样**写进 `<input type="hidden" name="userAgent" value="…">` ⇒ `"><script>alert()</script>` 可执行;但该头只能由**受害者自己的浏览器**提供 ⇒ 需走私把这条响应记到受害者的请求上。

## 外部形状(第三方 writeup,注明来源)

- Kettle「HTTP/1.1 Must Die / 0.CL」+ Brandon T. Elliott 改编的 Turbo Intruder 队列:纯 **HTTP/1.1**、靠 `Content-Length :`(空格)造成 0.CL 死锁,配 **early-response gadget**(静态路径)与**双重 desync**(`stage1` → `stage2_chopped`+`stage2_revealed`+`smuggled` → 受害者 `GET /`),走私请求 = `GET /post?postId=8` + `User-Agent: a"/><script>alert(1)</script>`;需反复重放。
  来源:`portswigger.net/blog/http-1-1-must-die-conquering-the-0-cl-challenge`、`brandon-t-elliott.github.io/0-cl-request-smuggling`(官方题页 solution 块未读)。
- 未收口面:批 44 的畸形头单发实验**未复现帧长分叉**,说明缺的是整套双 desync 编排(chopped/revealed 的精确字节算术),而不是某个载荷。

## 复现

```
conn_reuse <inst>/ --send-str 'POST /resources/css/anything HTTP/1.1\r\nHost: <inst>\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length : 92\r\n\r\nGET /404probe HTTP/1.1\r\nHost: <inst>\r\n\r\n'
```
