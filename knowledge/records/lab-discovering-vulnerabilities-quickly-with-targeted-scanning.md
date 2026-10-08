---
title: lab-discovering-vulnerabilities-quickly-with-targeted-scanning
---

# lab-discovering-vulnerabilities-quickly-with-targeted-scanning

> evidences: [[web-vuln-methods]]

- 题面:10 分钟内读取服务器任意文件(取 `/etc/passwd`)。shop 克隆。
- 判定:**stuck**(注入面、遍历面、反射面全部量化干净;真正的 file-read sink 仍未找到)

## 端点面

`/`、`/product?productId=N`(非整数 → 400 `"Invalid product ID: <原样回显>"`)、`POST /product/stock`、`/image/<path>`、未链接的 `/filter`(**无参数静态 listing**)、`/resources/*`、`/try-again`。产品页表单:`productId`(hidden)+`storeId`(select 1/2/3);`stockCheck.js` + `stockCheckPayload.js`(体 = `new URLSearchParams(data).toString()`,Content-Type urlencoded)。

- 首页静态:仅 20 个 `/product?productId=N` 链接 + 计时器(`deadline`、`/resources/js/labTimer.js`),**无搜索/评论/反馈面** ⇒ 「找反射面当扫描靶」不成立。
- 端点巡:`/filter` = 200(11261B)、`/product/stock` GET = 405;`/admin,/login,/register,/search,/feedback,/api,/robots.txt,/sitemap.xml,/cart,/checkout` 全 404。
- `stockCheckPayload.js` 只定义 `contentType='application/x-www-form-urlencoded'` 与 `URLSearchParams`;`stockCheck.js` 只做 fetch+重试。

## stock 端点 = 按串定值的 mock,不是文件读

- 对**任意** storeId 回一个 2-3 位数;同串稳定,同一路径的不同拼写各得各值(`/etc/passwd` 711、`/etc//passwd` 1、`/etc/passwd/` 755、`../../../../etc/passwd` 229)⇒ 值 = f(原串),**不是**文件系统读;值随实例变 ⇒ 每实例种子化的 mock。
- 无命令注入:`storeId=1;sleep 7` 无时延;`;id`、`1|cat …`、`$(cat …)`、`file://…`、`php://filter…` 全回数字。
- **只解析 urlencoded**:`Content-Type: application/json` 的 `{"productId":"1","storeId":"1"}` 与穿通形,以及 `application/xml` 的 `<stockCheck>…` 都落 400 `"No such product or store"`。
- storeId 含 NUL → 400;`GET /product/stock` → 405。

## productId 半

- 严格整数校验,两条 400 分支:回显分支 `"Invalid product ID: <原样>"` 收 `1'`/`1"`/`1;id`/`1.0`/`1 `(**原样未转义**进 `application/json` 体);固定分支 `"No such product or store"` 收含 `<`/`>`、`0`、`UNION` 形 ⇒ 反射面不构成 XSS。
- productId 走不到 SQL:任何非整数形都在校验层短路。
- 无 SSRF/无出网:storeId 取 `http://127.0.0.1:80/`、黑洞 `10.255.255.1:81`、`file:///etc/passwd` 全部瞬时返回 mock 数字。

## 遍历 / 头部面

- `/image` 前缀内 15 变体 + `/resources` 3 变体 + 8 种异形编码(`..;/`、`%2e%2e`、`....//`、`..%c0%af`、`..%5c`、双编码、`%00`、绝对路径、反斜杠、query `filename=`)全部 404;诚实 `/image/productcatalog/products/N.jpg` → 200 image/jpeg。
- 头部改写/反射:`X-Original-URL`、`X-Rewrite-URL`、`Referer`/`User-Agent` 带遍历 → 与基线逐字节相同。

## 计时器与实例寿命

- 10 分钟到点后所有路由回 `Time's up!`;`GET /try-again` → 302 重置。到点前后实例曾整体挂死数分钟自愈 ⇒ 记死为混杂证据,不得当注入证据。

## 复现命令

```
form_sweep <inst>/product/stock storeId '1,/etc/passwd,AAAAA,/etc//passwd,1;id,1|id' --field productId=1 --jar j
raw_matrix @traversal-spec.json
http_session post <inst>/product/stock --form storeId='1;sleep 10;'
```

## 关系

- 族:[[web-vuln-methods]];姊妹题 [[lab-scanning-non-standard-data-structures]];应用形状见 [[portswigger-burp-scanner-essential-skills-labs-app-shapes-and-traps]]。
