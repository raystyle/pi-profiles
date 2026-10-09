---
metadata:
  node_type: memory
name: "PRS lab solve infoleak on debug page"
description: "arm A 基线:lab-infoleak-on-debug-page 冷实例一次通过 - 首页 HTML 注释泄 /cgi-bin/phpinfo.php,SECRET_KEY 环境变量读出后 submitSolution correct:true,banner solved"
last_updated: 2026-10-09T06:51:09+08:00
created: 2026-10-09T06:51:09+08:00
---

## 2026-10-09 arm A 基线:lab-infoleak-on-debug-page

- 实例:range_launch launch-url(路径 `/web-security/information-disclosure/exploiting/lab-infoleak-on-debug-page`),reused:false,无 exploit server。
- 链路:
  1. `page_read` 取 widget-lab-id `18EBD0AC...9348` + 题面(目标=提交 `SECRET_KEY` 环境变量)。
  2. `range_launch launch-url <path> --jar /tmp/cj1.json` → 实例 `0a5a00f9...0037`。
  3. `http_session get /` → 首页末尾 HTML 注释泄露调试页:`<!-- <a href=/cgi-bin/phpinfo.php>Debug</a> -->`。
  4. `http_session get /cgi-bin/phpinfo.php --out /tmp/idp-phpinfo.html`(70200 B body)。
  5. `text_grep SECRET_KEY /tmp/idp-phpinfo.html` → 两处命中(line 599 环境段、line 632 `$_SERVER['SECRET_KEY']`),值 `2ztv6d5p9se3pkorm7lcm4zml92a119y`。
  6. `http_session post /submitSolution --form answer=<值>` → body `{"correct":true}`。
  7. `banner_verdict <base> --jar` → `solved:true` / `<h4>Congratulations, you solved the lab!</h4>`。
- 判定锚:banner `is-solved` + congrats 行;答案提交口无 CSRF,POST answer 直收。
- 关键形态:信息泄露族的入口是**站点自身资产里的隐蔽引用**(HTML 注释里的调试页链接),非目录爆破;debug 页(phpinfo)一次拉全,`SECRET_KEY` 同时出现在 Environment 段与 `$_SERVER` 段。
- 无新件需求:page_read/range_launch/http_session/text_grep/banner_verdict 已闭环。

