---
metadata:
  node_type: memory
name: "lab-file-upload-web-shell-upload-via-race-condition-A"
description: "arm A solve: upload_race piece landed the write-then-delete window on the file-upload race lab"
last_updated: 2026-10-10T02:42:54+08:00
created: 2026-10-10T02:42:54+08:00
---

- 2026-10-09, arm A solve, lab id E7C14C1093F823154152C9CE425959EF67D62C9BB7C65EB02F6A8A597AB993E8, instance 0a5200280445d39b8074ae1c009f000c.web-security-academy.net
- 路径: login wiener:peter (jar /tmp/cj1.json) -> 单发 shell.php 到 /my-account/avatar 得 403 (校验拒绝) -> 但文件在 /files/avatars/shell.php 仍可读,窗口比预期宽
- 工具: 新件 upload_race (.pi-rs/rust-scripts/upload_race.rs, 项目层) - U 个上传线程持续 POST multipart + G 个取回线程猛 GET 公开路径, 命中即停, 信封给两侧状态直方图/命中片段/密文候选; 首次跑 8 上传/24 取回 6s 内即命中, 23/24 个 GET 回 200 带 32 字符 secret
- 收尾: POST /submitSolution answer=<secret> 回 {"correct":true}, banner_verdict solved:true
- 经验: 上传校验竞态类题先单发一次探针(看拒绝码与路径), 再用 upload_race 并发; 该 lab 的写后删窗口实际很宽, 并发取回几乎立刻命中
