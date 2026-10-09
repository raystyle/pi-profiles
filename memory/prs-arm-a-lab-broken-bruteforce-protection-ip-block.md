---
metadata:
  node_type: memory
name: "PRS arm A lab-broken-bruteforce-protection-ip-block"
description: "arm A 基线:lab-broken-bruteforce-protection-ip-block 冷实例一次通过 - 每 2 个失败插一次 wiener:peter 诱饵登录重置 IP 计数,carlos:dragon 第 10 次命中,新增 login_brute 件"
last_updated: 2026-10-08T20:30:25+08:00
created: 2026-10-08T20:30:25+08:00
---

## 2026-10-08 arm A 基线评测:lab-broken-bruteforce-protection-ip-block

实例:`0a180010033c0c918026a8a200f80055.web-security-academy.net`(range_launch reused:false,冷实例;jar /tmp/cj1.json)。

链路(全部走件):

1. `page_read` 取 widget-lab-id `6D4CC1C4...1A`(题页解决方案块已剥离);`doc_read` 抓 https://portswigger.net/web-security/authentication/auth-lab-passwords 落 /tmp/pw.txt,候选口令在同一行代码块内(空格分隔,100 条),`text_grep` 定位后转写为 /tmp/pwlist.txt。
2. `range_launch` 起实例;`http_session get /login` 确认表单只有 username/password(无 CSRF)。
3. **新件 `login_brute`**(`.pi-rs/rust-scripts/login_brute.rs`,1.0.0,已 regen catalog):顺序口令爆破 + 诱饵重置。按 `--reset-every N` 个失败插入一次 `--decoy U:P` 的成功登录,把按 IP 计的失败计数清零;命中判据是 302 或响应体不含 `--fail-marker`;命中后写回 jar 并复验 `--confirm-path`。
   `login_brute <url> --user carlos --passwords /tmp/pwlist.txt --decoy wiener:peter --reset-every 2 --jar /tmp/cj1.json`
4. 结果:attempts=10、decoys=5、blocked_responses=0、elapsed 21.7s;第 10 次 `carlos:dragon` 回 302 `/my-account?id=carlos`。

证据:`banner_verdict` solved:true + `<h4>Congratulations, you solved the lab!</h4>`;带 jar 取 /my-account 得 `is-solved` 类与「Your username is: carlos」。

要点:该 lab 的封禁按 IP 计连续失败,任意一次成功登录(自有账号 wiener:peter)即清零,故诱饵登录必须真回 302 才算重置(件 tail 里诱饵行均 status 302 location=/my-account?id=wiener);开跑前先发一次诱饵,避免沿用会话里已有的失败计数。件走单线程顺序,重置语义要求请求严格串行。

