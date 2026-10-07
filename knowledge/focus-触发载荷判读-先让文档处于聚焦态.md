---
title: "focus 触发载荷判读:先让文档处于聚焦态"
---

# focus 触发载荷判读:先让文档处于聚焦态

# focus 触发载荷判读:先让文档处于聚焦态

判读任何**由 focus 事件触发**的载荷(`ng-focus`、`autofocus`、URL 片段 `#id` 聚焦、`onfocus`)之前,必须先确认目标文档 `document.hasFocus() === true`。

## 问题(实测)

无头/后台 harness 浏览器里:

- `document.activeElement` 会被 autofocus / 片段聚焦正确设成目标元素(看起来"触发成功"),
- 但 Chrome **不派发 focus 事件**(规范:文档未聚焦时 focus 只挂起),于是 `ng-focus` 处理器、`onfocus` 都跑不起来。

后果是**系统性假阴性**:批38 把 Angular CSP 题判成"片段聚焦早于 bootstrap"、批16 把 `javascript:` iframe 判成"不执行",都是这一类误读(后者另有一层:观测点只看 iframe 自身 document,而执行副作用在父页)。

## 正确姿势

1. 判读前:`browser_suite call Page.bringToFront`(必要时再 `call Emulation.setFocusEmulationEnabled --json '{"enabled":true}'`),然后在**同一 target** 里 `eval document.hasFocus()`;为 false ⇒ 结论作废。
2. 真阳性佐证:同一页面上 `zz=7` 这类副作用在**下一次** eval 里读到(ng-event 对 focus/blur 走 `$applyAsync`,同一次 eval 里读不到,会再吃一次假阴性)。
3. 在聚焦态下,URL 片段 `#x` + `id=x` 的**加载期**聚焦足以触发 `ng-focus` —— 不需要 `autofocus`(对 80 字符预算的题是决定性的)。
4. 跨源 iframe 里的片段导航:`iframe` 的 load 事件会对每次片段变更重放,但**子帧文档未聚焦时片段聚焦仍不生效**;此时可用网络 beacon(`window.alert` 打点 fetch 回 exploit server 访问日志)判读,别用只读顶层 `window.__labAlerts` 的件。

## 证据

```
eval: document.hasFocus() → false ; el.focus() 后 addEventListener('focus') 零命中
call Page.bringToFront → eval hasFocus → true
同一 autofocus+ng-focus 探针页: hasFocus=false 时 zz=undefined ; true 时 zz=7
page_alert "<79 字符载荷 URL>#x" : 聚焦态前 fired=false ; 聚焦态后 fired=true, alerts:["alert:"]
```

## 关系

- 工具面见 [[browse-cdp]];被此坑误判的题见 [[client-side-template-injection-family]](CSP 变体)与 [[dom-xss-family]](web message + javascript: iframe)。
