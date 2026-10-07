---
title: DOM XSS 族:源、汇与 web message
---

# DOM XSS 族:源、汇与 web message

同一类型漏洞:客户端 JS 把可控源(location/search/hash、postMessage)当数据
写进危险汇(innerHTML、document.write、`$()`、eval/JS URL)。与反射型不同,
流量里看不到载荷,须读部署代码定位源→汇。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| document.write 汇 | 脚本以 location.search 拼进标签 | 破引号注入 `<svg onload>` | [[158-dom-xss-docwrite]] |
| jQuery 选择器汇 | `$('…'+hash)` | hashchange 触发,`<img src=x onerror>` | [[lab-jquery-selector-hash-change-event]] |
| AngularJS 表达式 | ng-app + `{{ }}` 反射 | `{{$on.constructor('alert(1)')()}}` | [[lab-angularjs-expression]] |
| web message(innerHTML) | `message` 无 origin 校验写 `#ads.innerHTML` | 利用页 iframe + postMessage | [[lab-dom-xss-using-web-messages]] |
| web message(JS URL) | message 值进 `location`/JS URL | postMessage 携 `javascript:` | [[lab-dom-xss-using-web-messages-and-a-javascript-url]] |
| web message(JSON.parse) | message 经 JSON.parse 进 sink | 构造合法 JSON 让字段落 sink(未收口) | [[lab-dom-xss-using-web-messages-and-json-parse]] |
| DOM clobbering(净化器属性循环) | 净化器逐属性删、依赖 DOM 命名 | `<form id=x><input name=attributes>` 覆写属性 | [[lab-dom-clobbering-attributes-to-bypass-html-filters]] |
| DOM clobbering(变量覆写) | `let x = window.v || {默认}` 这类回退读取 | 同名 id 造 HTMLCollection + `<a name=avatar href="cid:&quot;onerror=…//">` | [[lab-dom-xss-exploiting-dom-clobbering]] |

## 共性

1. 先取部署 JS 清单(首页 `<script src>`)逐个人读,定位 `addEventListener('message')`/
   `document.write`/`innerHTML`/`$()` 的 dataflow;`browse eval '<fn>.toString()'` 最快。
2. `lab_alert --driver` 本地验证:直接调 sink 触发,勿让 `href` 跳转销毁文档(假阴性)。
3. 交付类用 exploit server:`<iframe ... onload="this.contentWindow.postMessage(...)">`;
   requests slug 404 时用 `/api/widgets` 解析真 slug。
4. clobbering 类必须让"覆写元素先入 DOM、危险读取后发生":评论按序渲染时首发 clobber、
   次发普通评论(本轮 clobber 只在下一轮迭代被读到);且回退值参与字符串拼接时
   元素的 `toString()` 即 href,是构造属性注入的支点。

## 判定与收尾要点

- 判定锚点:脚本在实例域实际执行(`fired:true` + `solved_check`);反射不算解。
- web message 题注意 `print()` 与 frame 关系;`--settle-ms` 给足。

## 相关族

- 反射型上下文逃逸见 [[xss-context-family]];原型链污染型 DOM XSS 见 [[prototype-pollution-family]];
  点击触发型见 [[clickjacking-family]];方法论:web-vuln-methods(seed 层,按名引用)。
