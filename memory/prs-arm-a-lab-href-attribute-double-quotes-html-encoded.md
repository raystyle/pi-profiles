---
metadata:
  node_type: memory
name: "PRS arm A lab-href-attribute-double-quotes-html-encoded"
description: "arm A 基线:lab-href-attribute-double-quotes-html-encoded 冷实例一次通过 - website 字段给 javascript:alert(1) 即 solved;range_launch 该 lab 返回 instance_url:null,改用 http_dump 读 launch URL 的 302 Location 取实例"
last_updated: 2026-10-09T00:02:17+08:00
created: 2026-10-09T00:02:17+08:00
---

题：/web-security/cross-site-scripting/contexts/lab-href-attribute-double-quotes-html-encoded，widget-lab-id `837B80F7B6CB679DDB307A046AF6DF7C851399E3337726434AC2681CBA79F639`。

结果：冷实例（首触 banner_verdict solved:false）一次通过 → congrats。链：page_read → 取实例 → http_session get / 取 postId=1 → get /post?postId=1 取 csrf → post /post/comment（website=javascript:alert(1)）→ banner_verdict solved:true。双引号被 HTML 编码，无需破属性，直接 javascript: 方案 URL。

环境坑（本会话实测）：
- range_launch 对该 lab 两次 `instance_url:null`，`final_url` 停在 `https://portswigger.net/web-security/`、`oidc_form_submitted:false`；同一 launch URL 用 `http_dump`（不跟随）直接见 302 → 实例 URL。`/users/youraccount` 走 Auth0 报 `invalid_request: couldn't find your session`（登录会话过期），但 launch 端点不需要 OIDC。结论：range_launch 失败时用 launch URL 的 `Location` 兜底取实例。
- /tmp/cj1.json 的 portswigger .AspNetCore cookies 仍可用于 launch；xlab 与 pi engine-profile 都没有 portswigger 会话 cookie。

