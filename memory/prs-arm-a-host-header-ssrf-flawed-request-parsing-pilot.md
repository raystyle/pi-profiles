---
metadata:
  node_type: memory
name: "PRS arm A host-header SSRF flawed request parsing pilot"
description: "深水重排臂 A(3 payload-forger 扇出 + 父臂器实射):题未解;实证平台给该 lab 加了反作弊——自供 _lab cookie 即 400 Too Nosy,家族旧配方失效,边缘把 Host 段锁成实例名 allowlist、请求行不参与路由"
last_updated: 2026-10-10T15:42:38+08:00
created: 2026-10-10T15:42:38+08:00
---

## 终态

`lab-host-header-ssrf-via-flawed-request-parsing`(实例 `0afe006a038ed4e0803f9eac006e004c`):**未解**。锚点 = `banner_verdict` → `{"solved":false,"congrats_line":null}`。前端面全程无 504、无内网面板、无可路由命中。

## 本臂编排(定制试点)

- 步骤①:族注 `host-header-family`(HH-3 记「请求行 absolute-form 与 Host 差异(未解)」)+ 全局 `subagent-orchestration-playbooks`(深水重排题 = 先族注复盘,再按面委派)。
- 步骤②扇出 3 个 payload-forger(纯构造、不投递;各返回变体表 + 粘贴用原字节 + 发射序):
  - `hh3-abs` 绝对形请求行族 19 变体(首推 V17 `http://<IP>@<inst>/admin`、V12 `\@` 形);
  - `hh3-host` Host 变形族 40 变体(首推 B07/B06 重复 Host 分裂、B26 尾点、B23/B24/B25 进制形);
  - `hh3-path` 路径内嵌授权族 28 变体(首推 C01 `//<IP>/admin`、C27 绝对形 + 协议相对路径)。
- 三子均报同一缺口:件面没有「只构造不发送」的请求字节装配器(raw_http/raw_matrix/abs_sweep/conn_reuse/h2_req 全都会发;cred_matrix 需既有模板),子无写文件工具故无法就地落地。
- 步骤③父臂实射:`raw_matrix`(每发独立连接)+ `conn_reuse`(字节级 + 全响应头)+ `h2_req`(原帧/伪头),全部经件,零手搓 HTTP。子首推候选全部覆盖到。

## 机制实证(本臂新得,已回填族注)

1. **`_lab` 反作弊**:自供 `_lab` cookie → `400 Client Error: Too Nosy`(170B,「Tampering with the _lab cookie is not required to solve this challenge.」);只发 `session` 或不发 cookie 均 200。⇒ 家族旧法(`Host: <内网 IP>` + `_lab`)在本实例直接失效,极可能就是前臂(trace 口径 47 次 rs_execute)全 4xx 的根因;调用数一律以 trace.sh rs_execute 计,勿用裸提及行数。
2. **边缘预检 1**:`Host` 头首个 `:` 前段必须等于实例主机名。公开域名(example.com、Collaborator 域名)与私网等价形(十进制/十六进制/八进制/短形/尾点/IPv6-mapped/大小写/@userinfo/带端口)一律 403 109B,**无 Set-Cookie**(层界判据)。
3. **边缘预检 2**:请求行 absolute-form 的 authority,name 形必须等于实例主机名;未识别形(百分号编码、`0300.0250.0.0252`、`[::ffff:…]`、裸 authority、`//` 形)放行。403 时**带 `_lab`、无 `session`** ⇒ 已过预检 1、死在请求行层。
4. **路由归属**:靶场前端只用 Host 头 host 段路由。请求行指向内网/等价形/Collaborator → 全部回电商 app(404 `"Not Found"` 11B 或首页 10691B),从不 504。以不可路由 IP `10.0.0.1` 与 `192.168.0.170/.171` 同形对照,响应逐字节相同 ⇒ 前端忽略 `@` 后缀,只取首个 `:` 前段。
5. 无转发头 fallback(X-Forwarded-Host/Forwarded/X-Forwarded-Server 均不生效);重复 Host(含大小写、`Host ␣` 变体)400;h2 的 `Host` 覆盖 `:authority`、大写 `Host`、双 `:authority` 均 GOAWAY;连接复用第二帧无响应(每响应后关连接,连接态攻击不适用)。

## 消耗与教训

- 约 85 发字节级请求、约 25 次件调用,无任何可路由命中。
- 教训 1:**归因先于枚举**。早期 4 发全是 400,差点当成「形状被拒」;读 body 才看到 `Too Nosy` 文案。此后每次矩阵都读 Set-Cookie 名做层界归因(预检 1 失败 = 无 cookie),效率立刻提升。
- 教训 2:**族注可能过期**。族注把 `_lab` 当钥匙,现实例已禁;族注与实录需带「反作弊版本」语境,否则复攻臂照着死方子打。
- 教训 3:构造型子代理能产出变体空间与发射序,但**没有可复用的构造件**——下批值得补 `reqline_forge`(变体表 → 转义请求字节,零 socket,带 CRLF 自检),三个子独立报了同一缺口。

