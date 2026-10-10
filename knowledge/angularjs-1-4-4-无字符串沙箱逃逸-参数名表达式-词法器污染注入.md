---
title: "AngularJS 1.4.4 无字符串沙箱逃逸(参数名表达式 + 词法器污染注入)"
---

# AngularJS 1.4.4 无字符串沙箱逃逸(参数名表达式 + 词法器污染注入)

## 适用面

AngularJS 1.4.x(client-side template injection 沙箱逃逸),注入点是一个被 `$parse` 当表达式编译的**字符串**;当引号被服务端实体化(术语见下)而无法写字面字符串、`$eval` 又不可达时用本式。

## 服务端形状

应用把每个查询参数写成内联 JS:

```js
var key = '<name>'; $scope.query[key] = '<value>'; $scope.value = $parse(key)($scope.query);
```

- 参数 NAME 就是 AngularJS 表达式,由 `$parse` 编译执行;作用域是 `$scope.query`(纯对象),VALUE 成为该对象上的字符串属性。
- NAME/VALUE 里的 `' " < > &` 被 HTML 实体化(`&apos;` `&quot;` `&lt;`),因此在 `<script>` 内它们是字面文本 —— 表达式里写不出引号。这就是 "without strings" 的成因。
- 参数求值顺序由服务端按名字哈希集序决定,不是 URL 序也不是字典序;单参数方案需要顺序可控,否则要在 URL 里放多个毒化参数 + 多个注入参数,保证至少一个毒化先于一个注入。

## 沙箱(1.4.4)边界

- `ensureSafeMemberName` 只拦 `__defineGetter__/__defineSetter__/__lookupGetter__/__lookupSetter__/__proto__`。
- `ensureSafeObject` 拦 `b.constructor===b`(即 Function 本体)、`b===Object`、`b.window===b`(window)、DOM 节点。故 `[].constructor.constructor` 直接 `[$parse:isecfn]`,且无法用 Object.defineProperty 改写。
- `ensureSafeFunction` 拦 Function 本体与 `Function.prototype.call/apply/bind`。
- 结论:走 `$parse` 常规路径拿不到 Function/window,必须换面。

## 逃逸式(实证)

1. **污染词法器**:`Lexer.lex`/`readIdent` 用 `String.prototype.charAt`。用一个参数名执行单表达式(无引号)
   `toString().constructor.prototype.charAt=[].join`
   之后 `charAt(i)` 变成 `Array.prototype.join.call(text, i)` —— 返回值以 `text[0]` 开头,于是 `isIdent(ch)` 恒真、`isNumber(ch)` 恒假,`readIdent` 把整段名称吞成**一个标识符**。
2. **代码注入**:编译器(CodeGen)把该标识符名原样拼进 `new Function` 的函数体(`s.<NAME>`、`l.<NAME>`、`ensureSafeObject(v3.<NAME>,text)`、`v0=v3.<NAME>=v1`)。注入名要在四个上下文里同时合法:用逗号表达式,末位留一个可赋值标识符。
3. **载荷**:`s,alert(1),s` 生成 `v5=s.s,alert(1),s;`,函数形参是 `(s,l,a,i)`,`s` 有定义,且 `alert` 在 `new Function` 的作用域链上直接是全局 `alert`,命中即弹窗。

## 判据

`page_alert` 报 `fired=true` / `alerts:["alert:1"]`;`banner_verdict` 报 `solved=true` 且 `<h4>Congratulations, you solved the lab!</h4>`。
