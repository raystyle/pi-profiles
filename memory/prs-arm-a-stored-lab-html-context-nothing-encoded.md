---
metadata:
  node_type: memory
name: "PRS arm A stored-lab-html-context-nothing-encoded"
description: "arm A baseline: stored-lab-html-context-nothing-encoded first-pass solve - workspace comment POST /post/comment with csrf+postId, payload &lt;script&gt;alert(1)&lt;/script&gt;, banner is-solved"
last_updated: 2026-10-08T23:12:14+08:00
created: 2026-10-08T23:12:14+08:00
---

## 2026-10-08 arm A baseline - stored XSS HTML context

题:Stored XSS into HTML context with nothing encoded(canonical 路径
/web-security/cross-site-scripting/stored/lab-html-context-nothing-encoded;
给定 slug 形 `stored-lab-...` 不是 academy 路径,page_read 直试才命中)。

链(一次通过,reused:false 冷实例)
1. page_read 确认 canonical 路径 -> widget-lab-id
   09E0BEE59E32E92A78E614589793110F95C397459ED0AB96A3DBAF100E3B7CDE。
2. range_launch 该 id + jar /tmp/cj1.json -> 实例 base。
3. http_session get 首页 -> /tmp/lab-home.html,search_content 抓 postId
   (4/6/1/9/3),取 /post?postId=1。
4. http_session get 该 post -> search_content 抓表单:
   action=/post/comment method=POST urlencoded;字段 csrf / postId /
   comment / name(required)/ email(required)/ website(pattern http)。
5. http_session post /post/comment 带 csrf+postId=1+
   comment=<script>alert(1)</script>+name/email/website。
6. 302 -> /post/comment/confirmation?postId=1 = 入库。
7. banner_verdict base+jar -> solved:true + congrats 行。

证据点
- post 页重取(search_content "probe")确认评论渲染;评论体因
  http_session 信封剥 script 块在落盘副本中看不到,不是未存储。
- 存储型 XSS 的判分为 banner 自身(受害者 bot 触达即翻),无需自跑浏览器。

坑
- 给定 slug 是简写,勿直接当 URL;canonical 需 page_read 试探或查列表。
- http_session 保存体剥 <script>,核对反射/落地必须用其它信标(如评论作者名)。

件:page_read / range_launch / http_session / search_content / banner_verdict,
无需新件。

