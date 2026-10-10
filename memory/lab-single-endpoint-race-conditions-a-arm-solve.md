---
metadata:
  node_type: memory
name: "Lab Single-Endpoint Race Conditions - A arm solve"
description: "A-arm solve of the single-endpoint race lab: leak model, async-mailbox polling, raceclaim piece, /admin delete"
last_updated: 2026-10-10T08:03:11+08:00
created: 2026-10-10T08:03:11+08:00
---

## 2026-10-09 - PortSwigger lab: Single-endpoint race conditions (无 solution 面读取, 自解)

目标: 把 carlos@ginandjuice.shop 绑到自己账号 (wiener), 再进 /admin 删 carlos。

机制(实测推得):
- POST /my-account/change-email (email, csrf) 每个请求都会发确认邮件, 收件人恒为账号当前邮箱
  (我方 exploit 域), 正文写的是该请求提交的新地址; 链接里的 token 与「待确认地址」都取自
  服务端共享的待变更记录, 所以在并发下会出现「正文 carlos + token 属于另一请求」的泄漏邮件。
- 该泄漏邮件就是我方能读到的 carlos 确认链接; 最后一笔写入的 token 才是活 token, 因此只有
  「泄漏邮件的 token 恰为最后写入」时确认才会成功。
- 邮件是异步投递: 一次 burst 的确认信会在随后 10-40 秒内陆续到达, 立即读收件箱只会看到上一轮
  的 token (这是我最初反复判 invalid 的原因)。轮询收件箱 + 只兑换正文为 carlos 的 token
  才有效; 先兑换任何 wiener 正文 token 会清掉待变更状态, 把 carlos 的机会烧掉。

工作流(件化: .pi-rs/rust-scripts/raceclaim.rs, 项目层):
raceclaim <inst> --target carlos@ginandjuice.shop --own wiener@exploit-<id>.exploit-server.net
  --csrf TOK --jar jar --inbox https://exploit-<id>.exploit-server.net
  --owns 2 --targets 8 --rounds 4 --wait-secs 12 --wait-polls 4 --delete-user carlos
- burst: 线程各自 warm /login, Barrier 齐放后并发 POST change-email (own 少、target 多)。
- 每轮 burst 后轮询收件箱 4 次(间隔 12s), 只挑正文 = target 且未见过的 token 去
  GET /confirm-email?user=wiener&token=...; 200/302 且账号页含 carlos@ginandjuice.shop 即成功。
- 收尾: GET /admin (拿到 Admin panel), GET /admin/delete?username=carlos (该 lab 删除是 GET 链接,
  无 csrf 表单字段, 我件里按表单解析 csrf 因此 admin_receipt 报 csrf_missing - 不影响结果)。

实证: 第 3 轮 poll1 命中 accepted 8dF7DoEh40YcPjuC -> carlos@ginandjuice.shop;
/admin/delete?username=carlos 返回 302 -> /admin 横幅 is-solved + "Congratulations, you solved the lab!"。

坑:
- csrf 随会话轮换, 旧 csrf 让 burst 全 400 (本轮 1-2 轮即如此), 仍需用最新 csrf; 邮件延迟到账的
  旧 token 反而救了场。
- race_email 件默认把 instance 当收件箱 host, 必须显式 --inbox-host 指 exploit-server, 否则 tokens=0。
- 单发(非并发)请求不会泄漏: 收件人随请求自己的写入走, 只有并发交错时正文与收件人/ token 才错位。

