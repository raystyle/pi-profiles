---
title: 知识库索引:族与工具地图
---

# 知识库索引:族与工具地图

`.pi-rs/knowledge/` 的入口(MAP of content)。三层:本索引 -> **族笔记**(漏洞族,一段定位 + 手法矩阵) -> **实录**(`records/`,单题实证)。方法论根笔记 `web-vuln-methods` 在 seed 层(跨库不连,按名引用)。工具面另见下面的工具笔记。

## 漏洞族(21)

### 客户端 / 前端

- [[xss-context-family]] — 反射型 XSS 的上下文逃逸:编码器探测、实体破引号、SVG/JS URL 绕过滤。
- [[dom-xss-family]] — DOM XSS 的源→汇:document.write、jQuery 选择器、AngularJS、web message、DOM clobbering。
- [[prototype-pollution-family]] — 客户端原型链污染:源(deparam/parseParams/BBQ)× 汇(script.src/eval/GA/descriptor)。
- [[clickjacking-family]] — 点击劫持:iframe 叠层、诱饵对齐、frame buster 绕。
- [[csrf-family]] — CSRF:SameSite(registrable domain)旁路与严格 CSP 下的同源表单劫持。
- [[oauth-family]] — OAuth:redirect_uri 路径穿越 + 代理页 postMessage 偷 fragment 令牌。

### 注入

- [[blind-injection-family]] — 盲注入:布尔条件响应 oracle 与 OOB 出网信道。
- [[prototype-pollution-family]] — 服务端原型链污染:无反射探测三 gadget、`constructor.prototype` 绕过滤、REST 参数污染。
- [[deserialization-family]] — 反序列化:PHP/Java 自研 gadget 链与 PHAR(polyglot)入口。
- [[ssrf-family]] — SSRF:黑/白名单过滤对齐与解析分歧绕过。
- [[host-header-family]] — Host 头族:鉴权、路由 SSRF、请求解析不一致、歧义请求缓存投毒。
- [[cache-poisoning-family]] — 缓存投毒与缓存欺骗:未键控面、参数遮蔽、fat GET、归一化、WCD。
- [[request-smuggling-family]] — HTTP/1.1 走私:CL.TE / TE.CL / TE.TE 帧构造与差分确认。
- [[h2-smuggling-family]] — HTTP/2 系走私:CRLF 注入、splitting、响应队列投毒、隧道。

### 认证 / 逻辑 / 竞态

- [[jwt-family]] — JWT:算法混淆(已知/未知公钥)与 RSA 模数反推。
- [[logic-flaws-family]] — 逻辑与访问控制:邮箱解析分歧与 IDOR/参数污染。
- [[race-conditions-family]] — 竞态:单/多端点、时间敏感令牌、部分构造;HTTP/2 单包齐发。
- [[graphql-family]] — GraphQL:端点、schema 暴露、私有字段与别名爆破。
- [[llm-attacks-family]] — LLM 攻击:间接提示注入、API 命令注入、过度代理。

### 基础设施 / 工具

- [[oob-callback-family]] — 自建 OOB 回调底座(DNS/HTTP)与 lab 出站实测边界。
- [[pentest-rs-port-family]] — pentest_rs 能力向 labkit 的移植映射与复用要点(工具族,非题解)。

## 工具笔记

- [[lab-launch]] — 靶场实例发射:widget 渲染与 OIDC 回放。
- [[browse-cdp]] — `cdp` crate 注入库:rs 件直连 CDP 的机制与核心 API。
- [[browse-dialect]] — `browse` 方言函数速查(导航/快照/交互/取值)。

## 约定

- 每份 `records/*.md` 的 frontmatter `links` 列出它实证的族(relation: evidences);正文顶部另有一行 `> evidences: <wiki 链接到族>` 形成 iwe 图边(record -> family)。
- 族笔记的「实录溯源」列出代表性实录(family -> record),双向可走。
- 未收口(stuck)题在实录与族笔记中均标注 `未收口` / `挂`。

