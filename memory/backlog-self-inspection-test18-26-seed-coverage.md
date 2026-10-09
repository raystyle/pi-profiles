---
metadata:
  node_type: memory
name: "Backlog Self-Inspection Test18-26 Seed Coverage"
description: "题18-26 回溯源@60494b5da(seed +25/-12 已提交):锁定窗口枚举/诱饵重置限速直达 P16(rank 2),cookie 离线爆破/token 不绑定账号不到(rank 8)→ 缺口候选 P25 离线凭证爆破、P26 令牌-主体未绑定(或补 P11 触发词)"
last_updated: 2026-10-08T20:59:34+08:00
created: 2026-10-08T20:59:34+08:00
---

## 2026-10-08 回溯自省台(题 18-26 九题欠账)@HEAD=60494b5da

版本面貌:HEAD 已从 ab232f2f1 推进到 60494b5da(gitlink bump labs 25+26;其间 118179a46/8ce829dab/d551c8826 等 lab gitlink bump)。seed tactical-patterns 相对 ab232f2f1 +25/-12 且已提交: P11 增 IDOR 子型(客户端可控维加「对象引用标识」,件组合加 `objref_scan`,并写明「标维 → 邻域枚举差分」);P13 增「周知路标(爬虫声明文件/接口文档)单 GET 直取(`http_session`)」+ 探测形留给未知披露面 + 字典爆破反例支;P14/P17/P20/P21/P22 把 `raw_matrix` 标注为「项目层件待晋 bundled」;P16 增转发头(X-Forwarded-For 类)可能不被记账的反例。P 条总数仍 24。

四查询实测(seed 根 universe=64,iwe find --lexical):
- 「锁定 窗口 枚举」→ tactical-patterns rank **2/23**(top: authentication-family>tactical-patterns),直达 P16(锁定检查先于认证 + 窗口标定)。
- 「诱饵 重置 限速」→ tactical-patterns rank **2/24**,直达 P16(计数重置条件与目标解耦;交错序压阈下 = 诱饵重置)。
- 「cookie 离线 爆破」→ tactical-patterns rank **8/27**,不到。
- 「token 不绑定 账号」→ tactical-patterns rank **8/25**,不到(top: authentication-family>cdp-interactive-debugging>request-forgery-family)。

缺口候选(seed 全文 grep:无「离线」、无「绑定」正向命中;仅 P13 反例支提到「字典爆破」):
1. **离线凭证爆破**(stay-logged-in cookie / base64(user:md5(pw))、hash 拼接口令、弱盐摘要)——P 层无条目。建议新增 **P25 可离线验证的凭证 -> 离线爆破**:判型=凭证本体可在本地自校验,验证不经服务端;机制=服务端限速/锁定全部无效,强度只取决于摘要算法 x 字典;件=`chrome_cookies`(离线解 cookie)+ 本地摘要字典爆破(新件或 sh 面)。
2. **令牌-主体未绑定**(重置/确认令牌有效但主体由请求参数决定,换 username 即夺号)——P 层无「绑定」字面。机制可归入 P11(客户端可控维),建议在 P11 触发词补「令牌/参数与主体未绑定、换主体重放」;**若判为独立机制**则新增 **P26 令牌与主体未绑定 -> 换主体重放**(件 `reqseq`/`http_session`)。

