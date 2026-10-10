---
title: AngularJS 1.4.x sandbox escape via expensiveChecks event-directive face and orderBy string predicate
---

# AngularJS 1.4.x sandbox escape via expensiveChecks event-directive face and orderBy string predicate

## 两个解析面

- expensive 面:事件指令(ng-focus / ng-click / ng-mouseover 等)在编译期调用 $parse(attr, null, true)。该面下每个成员读结果、每个调用实参与 callee context 都过 ensureSafeObject,因此 window 与 DOM 节点只要成为值即抛 isecwindow / isecdom。 - cheap 面:默认 $parse(过滤器内部对字符串谓词的解析、$eval、插值、ng-init 等)只对 blocked 成员名(如 constructor)加检查,成员读的值不被检查。

## 旁路链条

1. orderBy 过滤器接受字符串谓词,内部用默认(cheap)$parse 解析后按「数组元素作谓词 scope」调用。 2. $event.composedPath() 在 expensive 面可作为值(返回普通数组,不被值检查拦),其末项是 window,倒数第二项是 document。 3. 谓词以 window 为 scope 时,标识符 alert、document 解析为 window 上的同名属性。 4. callee 不能用成员调用形式 X.y(...):cheap 面对成员调用补 ensureSafeObject(context),window 作 context 会抛。改用非成员 callee 形状 (0||alert)(...):编译期只对 Identifier / MemberExpression / 赋值标识符 callee 生成 context,逻辑或条件 callee 走 l(args) 路径,无 context 检查。

## 形状

- 数组短形:$event.composedPath()|orderBy:'(0||alert)(document.cookie)' - 单元素形:[$event]|orderBy:'(0||view.alert)(view.document.cookie)'

## 与 CSP 的关系

CSP 为 script-src 'self'(无 unsafe-eval、无外部域)不阻断该路径:载荷是 DOM 属性里的 Angular 表达式,由同源加载的 Angular 自行编译执行,不需要内联 script、javascript: URL 或 eval。

## 触发面

事件谓词需要 $event,只能用事件指令;元素自身不必带 autofocus —— URL 片段 #<id> 会让浏览器聚焦该元素并派发 focus 事件,与 autofocus 等价且更省字符。

# AngularJS 1.4.x sandbox escape via expensiveChecks event-directive face and orderBy string predicate

## 两个解析面

- expensive 面:事件指令(ng-focus / ng-click / ng-mouseover 等)在编译期调用 $parse(attr, null, true)。该面下每个成员读结果、每个调用实参与 callee context 都过 ensureSafeObject,因此 window 与 DOM 节点只要成为值即抛 isecwindow / isecdom。 - cheap 面:默认 $parse(过滤器内部对字符串谓词的解析、$eval、插值、ng-init 等)只对 blocked 成员名(如 constructor)加检查,成员读的值不被检查。

## 旁路链条

1. orderBy 过滤器接受字符串谓词,内部用默认(cheap)$parse 解析后按「数组元素作谓词 scope」调用。 2. $event.composedPath() 在 expensive 面可作为值(返回普通数组,不被值检查拦),其末项是 window,倒数第二项是 document。 3. 谓词以 window 为 scope 时,标识符 alert、document 解析为 window 上的同名属性。 4. callee 不能用成员调用形式 X.y(...):cheap 面对成员调用补 ensureSafeObject(context),window 作 context 会抛。改用非成员 callee 形状 (0||alert)(...):编译期只对 Identifier / MemberExpression / 赋值标识符 callee 生成 context,逻辑或条件 callee 走 l(args) 路径,无 context 检查。

## 形状

- 数组短形:$event.composedPath()|orderBy:'(0||alert)(document.cookie)' - 单元素形:[$event]|orderBy:'(0||view.alert)(view.document.cookie)'

## 与 CSP 的关系

CSP 为 script-src 'self'(无 unsafe-eval、无外部域)不阻断该路径:载荷是 DOM 属性里的 Angular 表达式,由同源加载的 Angular 自行编译执行,不需要内联 script、javascript: URL 或 eval。

## 触发面

事件谓词需要 $event,只能用事件指令;元素自身不必带 autofocus —— URL 片段 #<id> 会让浏览器聚焦该元素并派发 focus 事件,与 autofocus 等价且更省字符。

# AngularJS 1.4.x sandbox escape via expensiveChecks event-directive face and orderBy string predicate

## 两个解析面

- expensive 面:事件指令(ng-focus / ng-click / ng-mouseover 等)在编译期调用 $parse(attr, null, true)。该面下每个成员读结果、每个调用实参与 callee context 都过 ensureSafeObject,因此 window 与 DOM 节点只要成为值即抛 isecwindow / isecdom。 - cheap 面:默认 $parse(过滤器内部对字符串谓词的解析、$eval、插值、ng-init 等)只对 blocked 成员名(如 constructor)加检查,成员读的值不被检查。

## 旁路链条

1. orderBy 过滤器接受字符串谓词,内部用默认(cheap)$parse 解析后按「数组元素作谓词 scope」调用。 2. $event.composedPath() 在 expensive 面可作为值(返回普通数组,不被值检查拦),其末项是 window,倒数第二项是 document。 3. 谓词以 window 为 scope 时,标识符 alert、document 解析为 window 上的同名属性。 4. callee 不能用成员调用形式 X.y(...):cheap 面对成员调用补 ensureSafeObject(context),window 作 context 会抛。改用非成员 callee 形状 (0||alert)(...):编译期只对 Identifier / MemberExpression / 赋值标识符 callee 生成 context,逻辑或条件 callee 走 l(args) 路径,无 context 检查。

## 形状

- 数组短形:$event.composedPath()|orderBy:'(0||alert)(document.cookie)' - 单元素形:[$event]|orderBy:'(0||view.alert)(view.document.cookie)'

## 与 CSP 的关系

CSP 为 script-src 'self'(无 unsafe-eval、无外部域)不阻断该路径:载荷是 DOM 属性里的 Angular 表达式,由同源加载的 Angular 自行编译执行,不需要内联 script、javascript: URL 或 eval。

## 触发面

事件谓词需要 $event,只能用事件指令;元素自身不必带 autofocus —— URL 片段 #<id> 会让浏览器聚焦该元素并派发 focus 事件,与 autofocus 等价且更省字符。

# AngularJS 1.4.x sandbox escape via expensiveChecks event-directive face and orderBy string predicate

## 两个解析面

- expensive 面:事件指令(ng-focus / ng-click / ng-mouseover 等)在编译期调用 $parse(attr, null, true)。该面下每个成员读结果、每个调用实参与 callee context 都过 ensureSafeObject,因此 window 与 DOM 节点只要成为值即抛 isecwindow / isecdom。 - cheap 面:默认 $parse(过滤器内部对字符串谓词的解析、$eval、插值、ng-init 等)只对 blocked 成员名(如 constructor)加检查,成员读的值不被检查。

## 旁路链条

1. orderBy 过滤器接受字符串谓词,内部用默认(cheap)$parse 解析后按「数组元素作谓词 scope」调用。 2. $event.composedPath() 在 expensive 面可作为值(返回普通数组,不被值检查拦),其末项是 window,倒数第二项是 document。 3. 谓词以 window 为 scope 时,标识符 alert、document 解析为 window 上的同名属性。 4. callee 不能用成员调用形式 X.y(...):cheap 面对成员调用补 ensureSafeObject(context),window 作 context 会抛。改用非成员 callee 形状 (0||alert)(...):编译期只对 Identifier / MemberExpression / 赋值标识符 callee 生成 context,逻辑或条件 callee 走 l(args) 路径,无 context 检查。

## 形状

- 数组短形:$event.composedPath()|orderBy:'(0||alert)(document.cookie)' - 单元素形:[$event]|orderBy:'(0||view.alert)(view.document.cookie)'

## 与 CSP 的关系

CSP 为 script-src 'self'(无 unsafe-eval、无外部域)不阻断该路径:载荷是 DOM 属性里的 Angular 表达式,由同源加载的 Angular 自行编译执行,不需要内联 script、javascript: URL 或 eval。

## 触发面

事件谓词需要 $event,只能用事件指令;元素自身不必带 autofocus —— URL 片段 #<id> 会让浏览器聚焦该元素并派发 focus 事件,与 autofocus 等价且更省字符。

# AngularJS 1.4.x sandbox escape via expensiveChecks event-directive face and orderBy string predicate

## 两个解析面

- expensive 面:事件指令(ng-focus / ng-click / ng-mouseover 等)在编译期调用 $parse(attr, null, true)。该面下每个成员读结果、每个调用实参与 callee context 都过 ensureSafeObject,因此 window 与 DOM 节点只要成为值即抛 isecwindow / isecdom。 - cheap 面:默认 $parse(过滤器内部对字符串谓词的解析、$eval、插值、ng-init 等)只对 blocked 成员名(如 constructor)加检查,成员读的值不被检查。

## 旁路链条

1. orderBy 过滤器接受字符串谓词,内部用默认(cheap)$parse 解析后按「数组元素作谓词 scope」调用。 2. $event.composedPath() 在 expensive 面可作为值(返回普通数组,不被值检查拦),其末项是 window,倒数第二项是 document。 3. 谓词以 window 为 scope 时,标识符 alert、document 解析为 window 上的同名属性。 4. callee 不能用成员调用形式 X.y(...):cheap 面对成员调用补 ensureSafeObject(context),window 作 context 会抛。改用非成员 callee 形状 (0||alert)(...):编译期只对 Identifier / MemberExpression / 赋值标识符 callee 生成 context,逻辑或条件 callee 走 l(args) 路径,无 context 检查。

## 形状

- 数组短形:$event.composedPath()|orderBy:'(0||alert)(document.cookie)' - 单元素形:[$event]|orderBy:'(0||view.alert)(view.document.cookie)'

## 与 CSP 的关系

CSP 为 script-src 'self'(无 unsafe-eval、无外部域)不阻断该路径:载荷是 DOM 属性里的 Angular 表达式,由同源加载的 Angular 自行编译执行,不需要内联 script、javascript: URL 或 eval。

## 触发面

事件谓词需要 $event,只能用事件指令;元素自身不必带 autofocus —— URL 片段 #<id> 会让浏览器聚焦该元素并派发 focus 事件,与 autofocus 等价且更省字符。
