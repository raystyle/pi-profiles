---
title: lab-discovering-vulnerabilities-quickly-with-targeted-scanning
---

# lab-discovering-vulnerabilities-quickly-with-targeted-scanning

> evidences: [[web-vuln-methods]]

- 题面:10 分钟内读取服务器任意文件(取 `/etc/passwd`)。shop 克隆。
- 实例(批51):https://0a8d002a040b240a83b369b7002e003c.web-security-academy.net
- 判定:**stuck**(注入面与遍历面全部量化干净;真正的 file-read sink 仍未找到)

## 端点面

`/`、`/product?productId=N`(非整数 -> 400 `"Invalid product ID: <原样回显>"`)、`POST /product/stock`、`/image/<path>`、未链接的 `/filter`(静态 listing)、`/resources/*`、`/try-again`。产品页表单:`productId`(hidden)+`storeId`(select 1/2/3);JS `stockCheck.js` + `stockCheckPayload.js`(体 = `new URLSearchParams(data).toString()`,Content-Type urlencoded)。

## stock 端点 = 按串定值的 mock,不是文件读

- 对**任意** storeId 回一个 2-3 位数;同串稳定(`1` -> 877 跨多次调用不变),同一路径的不同拼写各得各值(`/etc/passwd` 711、`/etc//passwd` 1、`/etc/passwd/` 755、`../../../../etc/passwd` 229)⇒ 值 = f(原串),**不是**文件系统读;值随实例变(批50 同端点 732/32)⇒ 每实例种子化的 mock。
- 无命令注入:`storeId=1;sleep 7;` 总耗时 2.3s vs 基线 2.2s;`storeId=1;sleep 10;` 亦 2.3s(`;sleep 7;#` 曾一次 6.4s,判噪声)。
- XML 体(正确 CL 的 `<stockCheck>…` 与带 DOCTYPE 实体的 XXE 体)一律 400 `"No such product or store"`;storeId 含 NUL -> 400;`GET /product/stock` -> 405。

## 遍历 / 头部面(批51 逐一量化)

- `/image` 前缀内 15 变体 + `/resources` 3 变体 + 8 种异形编码(`..;/`、`%2e%2e`、`....//`、`..%c0%af`、`..%5c`、双编码、`%00`、绝对路径、反斜杠、query `filename=`)全部 404 `"Not Found"`;诚实 `/image/productcatalog/products/6.jpg` -> 200 image/jpeg(`Cache-Control: public, max-age=3600`)。
- 头部改写/反射:`X-Original-URL`、`X-Rewrite-URL`、`Referer`/`User-Agent` 带遍历 -> 响应与基线逐字节相同(digest 8c2ac1c48e72b35f / 421088cf980b0e1f)。

## 计时器与实例寿命(批51)

- 10 分钟到点后**所有**路由回 `Time's up!`;`GET /try-again` -> 302 `/` 重启时钟。
- 到点前后实例曾整体不可达约 5 分钟(全请求挂死)后自愈 ⇒ 与「阻塞型 sink 拖死 worker」不可区分,记死为**混杂证据**,不得当注入证据。

## 复现命令

```
form_sweep <inst>/product/stock storeId '1,/etc/passwd,AAAAA,/etc//passwd,1;id,1|id' --field productId=1 --jar j
raw_matrix @traversal-spec.json        # 15+8 变体,marker root:
http_session post <inst>/product/stock --form storeId='1;sleep 10;'   # 计时判注入
```

## 关系

- 族:[[web-vuln-methods]];姊妹题 [[lab-scanning-non-standard-data-structures]];应用形状见 [[portswigger-burp-scanner-essential-skills-labs-app-shapes-and-traps]]。
