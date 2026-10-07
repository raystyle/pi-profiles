---
title: lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling
---

# lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling

> evidences: [[h2-smuggling-family]]

PortSwigger `request-smuggling/advanced/request-tunnelling/lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling`。批 44 实例 `0a9b007603f351cb80195378007f00d8`。目标:以 administrator 访问 `/admin` 并删 carlos。题面明确:**前端不复用后端连接**(只剩 tunnelling)。

- 判定:**stuck**(读通道与长度算术已量化;堵在"把前端追加的内部头读出来"这一步)。

## 批 44 新增证据(可复现的数值)

1. **读通道成立**:外层 `HEAD <path>` 的"预期长度"= 该 path 自身响应长 ⇒ h2 body 直接给出**嵌套响应原始字节**。
   - 外层 `HEAD /admin`(2776)⇒ body 里出现嵌套 `HTTP/1.1 401 Unauthorized … Content-Length: 2776` 全文(批 41 读到的"嵌套 401"其实就是这个,尺寸巧合)。
   - 外层 `HEAD /login`(3351)+ 嵌套请求长于它 ⇒ `500 Communication timed out`(可用作"嵌套响应 < 外层期望"的判据)。
2. **内部头泄漏的反射面与长度律**:嵌套 `POST / HTTP/1.1 … Content-Length: N\r\n\r\nsearch=` ⇒ 响应 R = **3389 + 转义后体的长度**,反射词落在**页尾**。
   - 实测:R=3589(CL 200,体全为 'b')、R=3709(CL 320)、R=3593(CL 60、`<`×48 ⇒ 每字符 +3,即 `&lt;` 4 字节转义)。
   - 即**体必须是 5338 字节左右**才能把 `R` 抬到外层 `/`(=8946)的期望长度之上,**同时**反射内容又必须落在前 8946 字节里 ⇒ 需要 `E∈[5271,5367]`(E=转义后长度)而追加头块 A∈[56,96] 字节(实测 CL 70 通、CL 110 挂)⇒ 窗口恰等于 A,必须一次命中 A 的精确值。
3. **追加头块长度实测边界**:体 = `search=: ` + 值 + 追加头;CL 70 能完成、CL 110 挂 ⇒ A(前端追加头块)∈[56,96] 字节。
4. **应用侧 `Keep-Alive: timeout=0`**:嵌套响应之后后端关连接 ⇒ "垫片请求凑长度"不可行(同 lab 4 的坑)。
5. 伪造头仍 401:外层 `HEAD /admin` + 嵌套 `GET /admin` 带 `X-SSL-VERIFIED: 1` + `X-SSL-CLIENT-CN: administrator`(不带 key)⇒ 401(未定 key 的存在与角色)。
6. 已知页面尺寸(本实例):`/`=8946、`/login`=3351、`/admin`=2776、`/resources/labheader/js/labHeader.js`=1515。

## 未决面

- 需要一次精确命中:`E = 5367 - A`、`count = E/4` 个 `<`、`CL = 9 + count + A`;由于 A 未逐字节测出(只知区间),要做 2~3 次二分(用"是否挂住"判定),再发一次大值(约 1.3K 字符)读泄漏。
- 拿到 `X-FRONTEND-KEY` 后仍不确定门是否只看 `X-SSL-CLIENT-CN`(第三方 writeup 说三者齐备即可;官方 solution 块未读)。

## 复现

```
h2_req <inst> --method HEAD --path /admin \
  --hdr2 'Teto: teto\r\n\r\nPOST / HTTP/1.1\r\nHost: <inst>\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 200\r\n\r\nsearch=||bbbb…'
```
