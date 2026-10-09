---
metadata:
  node_type: memory
name: "PRS arm A lab-deserialization-prebuilt-gadget-chain"
description: "arm A 基线:lab-deserialization-prebuilt-gadget-chain 冷实例一次通过 - phpinfo 泄 SECRET_KEY + 500 页泄 Symfony 4.3.6, phpggc Symfony/RCE4 载荷经新件 php_sess_forge 签名投递, banner solved"
last_updated: 2026-10-09T09:56:43+08:00
created: 2026-10-09T09:56:43+08:00
---

实例 reused:false, 一次通过到 congrats。链路:
1. range_launch launch-url 吃 canonical 路径直接起实例。
2. 首页注释泄 /cgi-bin/phpinfo.php → SECRET_KEY。
3. 会话 cookie = JSON `{"token": base64(serialized), "sig_hmac_sha1": hex}`,HMAC-SHA1 **覆盖 base64 串**(不是原始序列化字节; 对原始字节算得 2e99b08f 是错的)。
4. 框架识别靠应用自己的错误面: 用真密钥签一个非序列化 token 再访问, 500 页正文给 `Internal Server Error: Symfony Version: 4.3.6`。这条比猜框架可靠。
5. 选链按版本区间: phpggc Symfony/RCE4(3.4.0-34 / 4.2.0-11 / 4.3.0-7)覆盖 4.3.6; Symfony/FD1 覆盖不到 4.3.6。
6. 载荷 `php phpggc Symfony/RCE4 system 'rm /home/carlos/morale.txt'` 518B,含 NUL 私有属性名; 输出尾随 0x0a 必须 truncate; bash `$( )` 会吞 NUL,必须重定向落文件。
7. 新件 php_sess_forge v1.0.0(项目层): base64+HMAC-SHA1+JSON+PHP urlencode+写 jar, `--verify` 做阳性对照(必须从 jar/Set-Cookie 原样取 token,手抄必错), `--selftest` 走 RFC2202 向量。
8. 投递 http_dump --jar 首访即触发 __destruct → 500(链尾 `saveDeferred() on null` 致命属正常,命令已执行)→ banner_verdict solved:true。

记录文件: /tmp/ps_gadget/record-lab-deserialization-prebuilt-gadget-chain-A.md

