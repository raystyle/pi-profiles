---
title: "javascript: URL payload when parentheses are stripped"
---

# javascript: URL payload when parentheses are stripped

# javascript: URL 载荷在括弧被删时怎么写(throw + onerror + 强制转换)

Family: [[xss-context-family]]。触发纪律见 [[javascript-url-execution-needs-real-gesture]]。

## 问题

输入被反射进 `javascript:` URL 的字符串里,而服务端编码器**删除 `( ) [ ]`、反引号、反斜杠与所有 `%`**,只百分号编码 `' ; : = @ < > & ? +`(Chrome 真点击时会解码回原字符)。没有括弧就不能写 `alert(1)`,`new` / tagged template / `?.()` 也都被同一批字符封死。

## 手法:借用模板自带的调用点,把载荷做成追加实参

目标模板:

```js
fetch('/analytics', {method:'post',body:'<注入>'}).finally(_ => window.location = '/')
```

载荷(直接注进 `body` 字符串):

```
'},x=x=>{throw/**/onerror=alert,1337},toString=x,window+'',{x:'
```

- `'` 破串、`}` 闭合模板对象、`,` 给 `fetch` **追加实参** —— 括弧由模板自带,载荷里一个都不需要;
- 箭头体 `throw/**/onerror=alert,1337` 是逗号表达式:先把 `window.onerror` 指向 `alert`,再 `throw 1337`;
- `toString=x` 把 `window.toString` 换成该箭头;随后的 `window+''` 触发 ToPrimitive ⇒ 箭头被调用 ⇒ 抛出未捕获 `1337` ⇒ `onerror=alert` 收到 `Uncaught 1337`(消息可控且必含目标串);
- `/**/` 充当 token 分隔符:服务端把空格编成 `+`,href 里的 `+` 不会还原成空格。

## 通用要点

1. **先找模板自带的括弧**:注入点几乎总在某个字符串字面量里,而它外面已有调用(`fetch(`…`)`、`.finally(`…`)`、`.then(`…`)`);把载荷写成追加实参即可借到调用语法。
2. **控制 alert 的消息**用 `throw <值>` + `window.onerror=alert`——比 `alert('…')` 省掉全部引号与括弧,消息形如 `Uncaught <值>`。
3. **没有空格**时用 `/**/` 分隔 token。
4. **副作用式赋值当"调用点"**:`toString=x`、`onerror=alert` 这类全局属性赋值不需要括弧,随后由 ToPrimitive(字符串拼接/比较)隐式调用,是零括弧执行的关键。
5. 触发必须**真手势点击**;合成 `el.click()` 的负结果不是否证。