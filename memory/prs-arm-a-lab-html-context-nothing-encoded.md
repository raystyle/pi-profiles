---
metadata:
  node_type: memory
name: "PRS arm A lab-html-context-nothing-encoded"
description: "arm A 基线 lab-html-context-nothing-encoded 冷实例一次通过：page_read→range_launch→page_alert(fired)→banner solved；坑=http_session 信封剥 script 块导致反射点显示为空"
last_updated: 2026-10-08T22:07:52+08:00
created: 2026-10-08T22:07:52+08:00
---

arm A 基线：lab-html-context-nothing-encoded（反射 XSS，HTML 上下文无编码）冷实例（reused:false）一次通过。

- 链：`page_read` 取 widget-lab-id → `range_launch`（jar /tmp/cj1.json）→ `http_session get /?search=%3Cscript%3Ealert(1)%3C%2Fscript%3E` → `page_alert` 同 URL（fired:true, alerts:["alert:1"]）→ `banner_verdict` solved:true。
- 题页路径在 `/cross-site-scripting/reflected/` 下，`/contexts/` 版本 404。
- 坑：`http_session` 信封剥离 `<script>` 块，反射点显示成 `<h1>0 search results for ''</h1>`；不要把这种空引号误判为「未反射」。
- 实录落 `.pi-rs/knowledge/records/lab-html-context-nothing-encoded.md`（31 行）。无需新件。

