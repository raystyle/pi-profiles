---
metadata:
  node_type: memory
name: "Lab polyglot web shell upload arm A"
description: "A-arm solve: GIF89a 前缀 + PHP 多形体头像上传 RCE，读 /home/carlos/secret 并 /submitSolution 提交，banner solved"
last_updated: 2026-10-10T02:40:21+08:00
created: 2026-10-10T02:40:21+08:00
---

## 2026-10-09 lab-file-upload-remote-code-execution-via-polyglot-web-shell-upload (arm A)

- 实例: https://0a0f0081046859f781262fd100e5009a.web-security-academy.net/ (jar /tmp/cj1.json)
- 路径: page_read 题面 -> range_launch launch-url 起实例 -> 登录 wiener:peter (csrf 取自 /login)
- 关键观察/事件:
  - 上传面: POST /my-account/avatar, multipart 字段 avatar(文件)+user=wiener+csrf;账号页 csrf 与 change-email 表单共用。
  - 内容校验只认图片魔数级特征(非图像重编码)。35 字节真 GIF 不需要:纯 ASCII 载荷 GIF89a 前缀 + PHP 代码、扩展名 .php、Content-Type image/gif 即被接受(200, 空体)。
  - 验证载体: GET /files/avatars/poly.php 返回 GIF89a 前缀 + secret => PHP 执行且前缀字节原样输出。
  - 外传信道: 直读 /home/carlos/secret(echo file_get_contents),无 OOB 需求。
  - 提交面: POST /submitSolution form answer=<secret> -> {"correct":true};banner_verdict 得 Congratulations, you solved the lab!。
- 复用件: page_read / range_launch(launch-url,含取号) / http_session(登录、GET 验证、提交) / upload(多部件) / banner_verdict(翻牌锚点)。
- 教训: 上传类题的"真图校验"先按魔数假设验证一次(成本 1 请求),失败再升级为真图像+附加载荷,别一上手就造完整图像。

