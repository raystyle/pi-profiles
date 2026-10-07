---
title: 客户端模板注入族:AngularJS 表达式面与沙箱
---

# 客户端模板注入族:AngularJS 表达式面与沙箱

同一类型漏洞:用户输入落进 AngularJS 的**表达式/模板**求值面 → 表达式即代码。
判型先找"谁是模板、谁是沙箱、引号能不能用"。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| ng-app 内直接反射 | 搜索词原样落进 `<h1>{{…}}` 文本 | `{{$on.constructor('alert(1)')()}}`(1.7 无沙箱) | [[lab-angularjs-expression]] |
| 反射进脚本字符串 | `$scope.query[key]='<v>';` + `<h1>{{value}}</h1>` | `'`→`&apos;`、script 内不解码 → JS 串破不出 | [[lab-angular-sandbox-escape-without-strings]] |
| **参数名当表达式**(少见) | 每个 query 参数生成 `var key='<名字>'; … $parse(key)($scope.query)` | 参数名写成 Angular 表达式,结果落进 `{{value}}`(文本 sink) | [[lab-angular-sandbox-escape-without-strings]] |
| **CSP 变体** | `<body ng-app ng-csp>`,搜索词**原文**反射进 `<h1>`;`script-src 'self'` 挡内联 | `ng-focus` + `$event.composedPath()` + `orderBy` 字符串谓词 + `(y=alert)(document.cookie)` | [[lab-angular-sandbox-escape-and-csp]] |

## 共性

1. **先判引号编码**:`'`/`"` 是否实体化;实体在 `<script>` 里不解码 ⇒ 字符串字面量被废掉
   ("without strings" 类题面即源于此)。
2. **参数名/路径名也可能入表达式**——不要只盯参数值;同一页面多打几个参数就能看出服务端循环模板。
3. 结果面若是 `{{value}}` 文本插值,`{{7*7}}` 会**原样显示**(不会二次求值):sink 只在表达式求值处。

## Angular 1.4.4 沙箱速查(从 `angular_1-4-4.js` 读出并实测)

| 检查 | 拦什么 | 命中报错 |
|---|---|---|
| ensureSafeMemberName | `__proto__` `__define*/__lookup*Getter/Setter` | isecfld |
| ensureSafeObject | `b.constructor===b`(Function/Object 等构造器)、`b===Object`、`b.window===b` | isecfn / isecobj / isecwindow |
| ensureSafeFunction | 同上 + callee 为 `Function.prototype.call/apply/bind` | isecfn / isecff |

- 结论:`toString.constructor`、`[].filter.constructor`、`constructor`、`.call/.apply/.bind` 全被拦;
  可用的只剩**普通函数/实例方法**(如 `[].filter.toString()`、`(1+[]).constructor`=String、`String.fromCharCode`)。
- 表达式支持 `;` 多语句与赋值(`a=…;b=…`),`$parse` 的 scope 是**服务端传进来的数据对象**时无 `$eval`。

## CSP 变体的利用链

- **表达式预算**:先测搜索词/表单字串的**长度上限**(本 lab ≤ 80 字符,81+ → 400),载荷要在这个预算内塞下整条链。
- **expensiveChecks 是分水岭**:事件指令(`ng-focus` 等)走 `$parse(expr, null, **true**)`,
  标识符与成员访问的结果都被 `ensureSafeObject` 查 → `$event.view`(window)直接 `isecwindow`;
  而 `ng-init` 走 `scope.$eval`(非 expensive),且 **`orderBy` 的字符串谓词**由非 expensive 的
  `$parse(predicate)` 编译、**以数组元素为 scope** 求值——这是拿到「window 作用域」的唯一入口。
- **能过 expensive 检查的窗口获取式**:`$event.composedPath()`(返回**数组**,只查数组本身不查元素),
  路径里含 `document` 与 `window` 两项;`$event.path` 在现代 Chrome 已移除(实测 undefined)。
- **绕 isecwindow 的调用式**:普通 `alert(document.cookie)`(scope=window)被 context 检查拦住;
  **赋值式调用** `(y=alert)(document.cookie)` 可以 → alert 实发且参数就是真 cookie。
- **单字段坑(未解)**:80 字符下加不进 `autofocus`(需 84),本次用 `id=x` + URL `#x` 片段导航聚焦;
  实测弹出了真 `document.cookie`,但该变体 lab 判定未翻(见实录未决面)。

## 判定与收尾要点

- 判定锚点:浏览器内 `alert` 实发(`lab_alert`)→ `solved_check` true。
- 快速判定候选表达式不必真导航:页面里用 `angular.element(document.body).injector().get('$parse')`
  对**自造 scope 对象**求值,`String(result)` / 错误码即回执(oracle 法)。

## 相关族

- 反射编码器判读与 `{{}}` 面见 [[xss-context-family]];`lab_alert`/`browser_suite` 用法见 [[browse-cdp]]。
- 工具:内置 `lab_alert`(alert 勾子 + `--driver`)、`browser_suite`(goto/eval/call)。
