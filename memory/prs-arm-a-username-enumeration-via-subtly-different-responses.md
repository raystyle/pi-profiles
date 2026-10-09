---
metadata:
  node_type: memory
name: "PRS arm A username-enumeration-via-subtly-different-responses"
description: "arm A 基线:lab-username-enumeration-via-subtly-different-responses 一次通过(ftp/football);长度通道被随机 analytics 脚本污染,改用 cache_probe --marker 找异常行"
last_updated: 2026-10-08T20:08:25+08:00
created: 2026-10-08T20:08:25+08:00
---

## 2026-10-08 arm A 基线:lab-username-enumeration-via-subtly-different-responses

- 起实例:`range_launch launch 4EACBB...FE0D --jar /tmp/cj1.json` → reused:false,一次冷实例;banner 终判 solved=true。
- 页面信息:`page_read` 取 widget-lab-id;候选名单在 `/web-security/authentication/auth-lab-usernames`、`auth-lab-passwords`(各 101 / 100 条,HTML 里是单个 `<code>` 块,空格分隔)。
- 表单:`POST /login`,字段只有 username/password,**无 CSRF**;失败响应 200 + `<p class=is-warning>Invalid username or password.</p>`,成功 302 → `/my-account?id=<user>`。
- 结果:用户名 `ftp`(清单第 12 条),口令 `football`(清单第 14 条),登录后账户页 banner 转 is-solved。
- 关键陷阱(本题核心):**长度通道不可用**。响应体里含 `<script>fetch('/analytics?id=<随机9位数>')</script>`,并有随机多出的 `<!-- -->` 空行,同一请求两次体长 3336/3352 波动 ±16,而"subtly different"信号只有 1 个字符 ⇒ `form_sweep` 的 `len`、`http_dump` 的 `body_len` 判不出有效用户名。
- 可用判据:`cache_probe` 的多请求 + `--marker`(它会报每行 `marker.in_body` 布尔),以"无效用户消息里的 `password.`"为 marker,拿 101 个用户名各发一次 POST,唯一 `marker.in_body=false` 的就是有效用户名。一个信封收敛。
- 经验:`cache_probe` 单行回执冗长(400 字符 body + 全字段),101 行会把信封撑到截断丢头;实战用 `--body-limit 1` 仍偏大,可靠做法是**先按候选清单切小批**(15/50 条一段),或只看已确认有差异的那一段。
- 经验:spec 不必手写 —— 用 `text_sub` 三段式生成:`([a-z0-9][a-z0-9-]*)` → 请求对象、`,\s*\z` → 闭合、`\A` → 补 `{base,pragma:false,requests:[`(尾部要记得补外层 `}`)。
- 口令侧:`form_sweep` 用 `status`/`location` 判读(302 即命中),该 lab 无锁定/限速,101 条一次跑约 66 秒。

