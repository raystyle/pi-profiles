---
title: browse 方言函数速查
---

# browse 方言函数速查

browse 缺省形态求值「方言片段」:一段 JS 形语句(字面量/对象/数组/成员/
下标/`await`/`const-let-var`/`return`/反引号模板串),驱动常驻 clean-chrome。

## 值语言定位

- 无运算符、无 `if/for/while`(解析期即报错并提示改走 `Runtime.evaluate`)。
- 比较、运算、循环等页面内计算一律进 `pageEval` 的 JS 串做。
- 值只有少数纯方法:字符串 slice/split/includes/startsWith/endsWith/trim/
  大小写;数组 slice/join/includes/concat。比较是 JSON 深等(非 JS 引用等)。

## 导航

- `goto(url, opts?)` — navigate+waitLoad 一体收尾,回 {url,title,elapsedMs};
  opts.timeout 秒(缺省 15)、waitIdle:true 再等网络静默。
- `waitLoad(s?)` — 等 document.readyState=complete(已加载即返),缺省 10 秒。
- `waitFor(event, s?)` — 等 CDP 事件(如 frameNavigated)。
- `goBack(delta?)` / `goForward(delta?)` / `reload(opts?)` — 历史与刷新。
- `listPageTargets()` / `switchTab(id)` / `currentTab()` / `newTab(url?)` /
  `closeTab(id?)` — 多 tab。

## 取结构(引用表)

- `snapshot(opts?)` — AX 树快照,nodes 带 role/name/value/childIds/短 ref
  (e1、e2…),是 ref 表的唯一来源;opts.ref 取子树、depth 限深、pierce:true
  穿同源 iframe 与 shadow DOM。
- `findRefs(query, opts?)` — 服务端按 name/value 子串检索,只回命中加
  context 层祖先链(insensitive 开关);命中即整表换新 ref,零命中保留旧表
  (kept_refs:true)。比全量 snapshot 省一个量级 token。
- `semanticSnapshot(opts?)` — 语义快照(需扩展域版引擎)。

## 交互(ref 来自最近 snapshot/findRefs)

- `clickRef(ref, opts?)` — 滚动可见、量中心、遮挡命中测试、trusted 派发;
  opts.button/clickCount/waitNav(链接跳转后有界等待收尾)。
- `fillRef(ref, text, submit?)` — 填输入框,submit 顺带 Enter。
- `typeRef(ref,text)` / `hoverRef(ref)` / `dblclickRef(ref)` /
  `dragRef(src,dst)` / `checkRef` / `uncheckRef` / `selectOption(ref,value)` /
  `pressKey(key)`。
- `clickAt(x,y,opts?)` / `hoverAt(x,y)` / `mouseMove/mouseDown/mouseUp` /
  `mouseWheel(dx,dy)` / `dropFiles(ref,paths)` — 视口坐标与原生输入。
- `fillInput(selector, text, submit?)` — 按 CSS 选择器填。

## 取值与诊断

- `pageEval(js)` — 全量 JS 一次求值(Runtime.evaluate,returnByValue 加
  awaitPromise),任意页面内计算全走它;等价 `browse --js`。
- `waitForResponse(pattern, s?)` — 等 URL 命中 glob 的响应完成,回
  {requestId,url,status,headers,body,json}(Network 须在触发前已开;方言无
  并发,只能先触发后等);`responseBody(requestId)` 再取体。
- `requests(opts?)` / `requestDetail(idx,opts?)` — 请求清单与详情。
- `detect()` 环境七判;`console(opts?)` / `jsErrors(since?)` — 控制台与异常。

## 会话与存储

- `cookies(domain?)` / `cookieGet/cookieSet/cookieDelete/cookiesClear`、
  `localGet/Set/Delete/Clear`、`sessionGet/Set/Delete/Clear`。
- `grantPermissions(perms,origin?)`、`cloneCookies(domains)`、
  `emulate(opts)` / `emulateMedia(opts)`、`screenshot(path?,full?,opts?)` /
  `pdf(path?)`。
- 低阶面 `session.*`:`connect` / `use` / `call(method,params?)` /
  `waitFor(method,undefined,s?)` / `waitJs` / `peekEvents` / `findEvents`。
- `print(x)`、`JSON.parse(s)` / `JSON.stringify(v,indent?)`。

## 组合模式

- 导航取数:`goto(url)` 后 `pageEval(js)`;或启 Network 后
  `waitForResponse(pattern)` 再 `responseBody(requestId)`。
- 检索交互:`findRefs(q)` 得 ref 再 `clickRef`/`fillRef`;每次新快照会刷新
  ref,旧 ref 过期即报错,交互前先重取。
- 取页面链接:方言无独立 `links` 函数,用 `pageEval` 遍历
  `document.querySelectorAll('a')` 收集 href。

## 约定

- 错误回执自带可照抄的「下一步」,按提示补参或换函数即可续跑。
- 大页先 `snapshot({depth:1})` 浅扫,再 `findRefs` 或 `snapshot({ref})`
  部分展开,省 token。
