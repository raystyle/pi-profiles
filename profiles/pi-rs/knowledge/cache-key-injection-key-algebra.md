---
title: "cache-key-injection-key-algebra"
---

# cache-key-injection-key-algebra

# cache-key-injection-key-algebra

缓存键由"请求目标 + 分隔符 + 某个 keyed 头"拼接时的键代数与命中条件(实测于学院 `lab-web-cache-poisoning-cache-key-injection`)。

## 键构造

```
key = <request-target> + "$$" + (Origin 头存在 ? "Origin=<值>" : "")
```

- **Origin 头名大小写原样进键**:发 `origin:` → 键为 `$$origin=…`;发 `Origin:` → `$$Origin=…`。命中是字符串全等,所以若受害者的子资源 URL 里带的是**小写** `origin=`(例如用自己的 `$$origin=` 段注入进 script src),投毒请求就必须发小写头——这是最容易误判的一环。
- `utm_content=<v>&` 这一对被键正则剥掉(剥到下一个 `&`)⇒ 唯一"后端多看到参数、键保持不变"的注入口。
- 键用 `Pragma: x-get-cache-key` 回读;`http_dump`/`raw_http` 都不规范化头名大小写。

## 组合形状(两类独立缺陷拼一条链)

1. 找**可缓存 + 把请求头未转义反射进响应头**的端点(此处 `localize.js?cors=1` 把 `Origin` 反射进 ACAO,且 `%0d%0a` 会被解码)⇒ 注入 `Content-Length: N` + 空行 + 载荷体,得到"响应拆分式"投毒体。
2. 找**可缓存 302 且 Location 原样带未解码 query** 的端点(此处 `/login?lang=…`)⇒ 把注入后的值交给受害者,使其页面生成的子资源 URL 恰好等于第 1 步的键(配平手段:`#` 截断服务端追加的尾缀、`utm_content` 对被键剥离)。

## 纪律

- 先读键再设计载荷;不要用裸 `/` 校验 solved(会把干净页写回缓存)。
- TTL 短(该题 `max-age=35`):每个请求各起一个 `poison_loop`(间隔约 8s)续毒直到受害者命中。

## Links

- evidences: [[cache-poisoning-family]]
