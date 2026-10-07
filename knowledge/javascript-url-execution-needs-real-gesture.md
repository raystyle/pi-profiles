---
title: "javascript-url-execution-needs-real-gesture"
---

# javascript-url-execution-needs-real-gesture

# javascript-url-execution-needs-real-gesture

**判定 `javascript:` URL 是否执行、以及是否先做百分号解码,必须用真用户手势;合成点击(元素上直接 `el.click()`)不能作为否证依据。**

## 现象

同一 URL,两种"点击"结论相反:

| 点击方式 | `href="javascript:alert%281337%29"` |
| --- | --- |
| 合成:`a.click()`(页内 JS) | 不执行(被静默挡下,无任何异常) |
| 真手势:`page_alert --click`(CDP Input 管线) | **执行**,`fired=true, alerts:["alert:1337"]` |

⇒ **Chrome 在执行 `javascript:` URL 前会先做百分号解码**;之前"现代 Chrome 不解码"的结论来自合成点击的假阴性,并由此推出了整条错误的"双重编码换字面括弧"推理。

## 纪律

1. 任何"`javascript:` payload 不执行"的结论,先问:点击是合成的还是真手势?合成一律判**不确定**,升档到真输入管线(`page_alert --click`、`browser_suite` + `Input.dispatchMouseEvent`、`page_interact --click`)再判。
2. 同理适用于所有"需要用户激活"的行为:弹窗、导航、剪贴板、`window.open`。
3. 与 [[focus-触发载荷判读-先让文档处于聚焦态]] 同一类错误:**判读条件不满足时得到的负结果不是否证**。判读前先确认前置条件(焦点/手势/加载态),再下结论。

## 关系

- 家族:[[xss-context-family]](`javascript:` URL 射点行按此改写)、[[browse-cdp]](真输入管线的件面用法)。
