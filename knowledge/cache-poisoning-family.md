---
title: 缓存投毒族:键控、归一化与投毒链
---

# 缓存投毒族:键控、归一化与投毒链


同一类型漏洞:响应入缓存时,缓存键与源站解析/渲染所用的请求分量不一致,
攻击者污染一个键,受害者的干净请求命中被污染的条目。族内两分:投毒
(poison,页面/资源体被写脏)与 Web 缓存欺骗(WCD,用归一化把本不该缓存的
会话页存成静态路径再由匿名读回)。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| 未键控头 | 响应无 `Vary`,头值回显进页面/资源地址 | 投 `X-Forwarded-Host` 等,使页面外链指向利用服务器 | [[lab-web-cache-poisoning-with-an-unkeyed-header]] |
| 未键控 cookie | Cookie 不进缓存键、值回显进内联 JSON | cookie 值破引号注入 JS | [[lab-web-cache-poisoning-with-an-unkeyed-cookie]] |
| 多头合用 | 单头无反射,两头组合出缺陷 | `X-Forwarded-Scheme`+`X-Forwarded-Host` 造 302 并缓存 | [[lab-web-cache-poisoning-with-multiple-headers]] |
| 组合链 | 未键控头引 DOM/JSON 注入 | XFH 改 `data.host` -> 跨源 JSON 当 HTML | [[lab-web-cache-poisoning-to-exploit-a-dom-vulnerability-via-a-cache-with-strict-cacheability-criteria]]、[[lab-web-cache-poisoning-combining-vulnerabilities]] |
| 参数遮蔽 | 键排除某参数、后端把它当分隔符或写进响应态 | 参数遮蔽形投毒,落受害者键(具体已证形存项目层) | [[lab-web-cache-poisoning-param-cloaking]] |
| fat GET | GET 带 body,后端取 body、键只含 query | body 里放 `callback=` 改 JSONP | [[lab-web-cache-poisoning-fat-get]] |
| 归一化 | 键按解码后 URL 归一,源站按原文 | 原始字节投毒、编码 URL 交付 | [[lab-web-cache-poisoning-normalization]] |
| 缓存键注入 | 键含 `$$`/Origin 段 | 向键段注入以落他人键(未收口) | [[lab-web-cache-poisoning-cache-key-injection]] |
| 内部缓存 | 首页无缓存头,上游另有内部缓存 | 找内部缓存面(未收口) | [[lab-web-cache-poisoning-internal]] |
| 定向投毒 | `Vary: User-Agent`,需目标子集 | 先取受害者 UA,再按 UA 投毒(未收口) | [[lab-web-cache-poisoning-targeted-using-an-unknown-header]] |
| 缓存欺骗(WCD) | 缓存规则/归一化使会话页落静态路径 | 归一化差分把会话页存成静态路径(分隔符编码形存项目层) | [[lab-wcd-exploiting-exact-match-cache-rules]] |
| 未键控参数 | 键排除某参数,但其值进 canonical/Set-Cookie | 裸字节请求投毒(`raw_poison`),同键多轮续毒 | [[lab-web-cache-poisoning-unkeyed-param]] |
| 未键控整条 query | 整条 query string 不进键(任意 query 都映射到 `/`) | `/?<payload>` 裸字节投毒,多轮覆盖 TTL | [[lab-web-cache-poisoning-unkeyed-query]] |

## 共性

1. 三问先行:缓存键是什么、响应有哪些缓存头(`Cache-Control`/`Age`/`X-Cache`/`Vary`)、
   源站渲染又读哪些请求分量;差值即未键控面。
2. 键探测件:`http_dump`(全响应头,看 `X-Cache`/`Age`/`Vary`)、`Pragma: x-get-cache-key`
   读 `X-Cache-Key`;`url_fuzz` 探规则/分隔符;`header_fuzz` 逐头枚举找未键控反射。
3. 短 TTL 需持续投毒:`poison_loop`(定时重复)、`fatget_poison`、`raw_poison`(字节级请求行)。
4. 投毒后**紧跟**一个干净请求验证 `X-Cache: hit`;交付类用 exploit server
   交付表单指定投递给受害者,读投放日志确认。
5. unkeyed 组的反射面常见于自引用标签(canonical/og 等;属性引号形决定破法,题面级已证形存项目层 [[portswigger-platform-specifics]]),
   而"未键控"的佐证包括:被排除参数的值出现在响应态里(如 Set-Cookie)、只带该参数的请求命中根键。
   这类 lab 的 cache TTL 短(本批 `max-age 短窗`),payload 含 `'`/`<`/`>` 时**必须**用 `raw_poison` 发字节级请求行
   (ureq 会重新编码),并按 TTL 多轮续毒直到受害者命中(TTL 过期后先到者会写回干净页)。

## 判定与收尾要点

- 判定锚点:受害者浏览器在实例域实际执行 `alert(document.cookie)` / `print()`,
  或匿名读回受害者的会话页/敏感值(WCD);`X-Cache: hit` 只是中间证据。
- **别用裸根路径校验 solved**——会把干净页写回缓存挤掉投毒;校验读带随机参数的请求
  (见 [[lab-web-cache-poisoning-combining-vulnerabilities]])。
- WCD 的关键是缓存规则(精确文件名/目录)与源站截断字符(`;`、Tomcat)的差;
  投毒后须在**无会话**上下文读回。

## 相关族

- 头信任差分见 [[host-header-family]];请求行/后端解析差分见 [[request-smuggling-family]]。
- 方法论:web-vuln-methods(seed 层,跨库不连,按名引用)。

## WCD 五子型(缓存欺骗;2026-10-10 v2 轨 5/5 实解)

| 子型 | 机制句(五题实锚直引) |
|---|---|
| 路径定界符 | 缓存对 `;` 等定界符的路径截断与源站解析错位,分隔符后挂目标路径 |
| 缓存服务器归一 | 缓存服务器层对路径的规范化与源站不一致,消解差内穿 |
| 源站归一差 | 404 页逐字节回显路径(免 exploit server 的自检/投递面);缓存按前缀判定可缓存、源站只对小写 %2f 归一——`/resources/..%2fmy-account` 命中前缀规则却在源站解析为目标页;exploit server 投 document.location 让受害者首访造缓存条目,窗口内无 cookie 读回其账户页 |
| 后缀映射 | 静态后缀到动态路径的映射差,后缀形命中缓存键而源站走映射 |
| exact-match 严格规则 | 缓存规则按字面全匹配路径,逐字符探规则边界后在规则内嵌目标 |

## 族地板带(v2 轨 14/14)

- 全解带 10-46,中位 21;24 线内 9/14;**>24 尾皆多段链或双头/定向形**(path-delimiters 25/exact-match 30/unknown-header 33/combining 33/multiple-headers 46 全 not-smoke 走四台环)。
