---
metadata:
  node_type: memory
name: "PRS arm A lab-referer-validation-depends-on-header-being-present"
description: "arm A 基线:lab-referer-validation-depends-on-header-being-present 冷实例一次通过 - 无 Referer 头即跳过校验,meta no-referrer 自提交表单改 victim 邮箱,banner solved"
last_updated: 2026-10-09T07:42:10+08:00
created: 2026-10-09T07:42:10+08:00
---

## 2026-10-09 arm A 基线

- 取实例:`range_launch launch-url <canonical path>` 返回 instance_url=null(落回 web-security 首页);回退 `http_session get <launch_url> --follow` 驱动启动链,hop1 302 的 Location 即实例根 `0a57006903f0869480ee03e8003800cc.web-security-academy.net`;exploit server 地址在同页 HTML 的 `#exploit-link`(本轮 `exploit-0a6b00310372862680eb024c01940076.exploit-server.net`)。
- 面:登录 wiener:peter → /my-account;改邮箱面 `POST /my-account/change-email`,单字段 `email`,无 csrf token。
- 防御语义实测(两腿对照):不带 Referer → 302 且改邮箱生效;`Referer: https://evil.example/` → 400 body `"Invalid referer header"`。即校验以头存在为前提。
- 解法:利用"头缺失即不校验",exploit 页放 `<meta name="referrer" content="no-referrer">` 让 Chrome 提交时省略 Referer,表单自提交 POST 到实例的 /my-account/change-email,email=hacker@evil-user.net。
- 交付:STORE(responseFile=/exploit + responseHead + responseBody)→ GET /exploit 核实体已存 → DELIVER_TO_VICTIM 并 --follow;交付响应自身即带 is-solved 与 "Congratulations, you solved the lab!"。
- 坑:DELIVER_TO_VICTIM 的最小字段集(仅 formAction/urlIsHttps/responseFile)得 400 `"Missing parameter responseHead"`,须补齐 responseHead(与 responseBody)按 UI 全表单提交。
- 复核:banner_verdict `solved:true` / `solved_class:true`。
- 件:range_launch + http_session + banner_verdict 已足,无需新件;未 git 提交。

