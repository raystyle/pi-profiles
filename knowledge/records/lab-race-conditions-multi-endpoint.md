---
title: "lab-race-conditions-multi-endpoint"
links:
  - target: race-conditions-family
    relation: evidences
---

# lab-race-conditions-multi-endpoint

> evidences: [[race-conditions-family]]

- 题面:Multi-endpoint race conditions(/web-security/race-conditions/lab-race-conditions-multi-endpoint)
- 实例:https://0a2e002f04c14b638032306200d40074.web-security-academy.net
- 判定目标:以非预期价格买到 Lightweight "l33t" Leather Jacket($1337,信用 $100)
- 状态:**stuck**(超 15min 上限;未决面在本档)

## 已确认的机制

1. 店铺:`/cart`(加/减物,POST productId/quantity/redir)、`/cart/coupon`、`/cart/checkout`(csrf,校验+下单同请求)、
   `/gift-card`(兑换)、产品 1=夹克 $1337、2=Gift Card $10。
2. `/cart/checkout` 用**一次**读到的购物车做支付校验与下单:夹克在车里 → `303 /cart?err=INSUFFICIENT_FUNDS`;便宜车 → `200` 订单确认页。
3. 购物车是**按会话**的:A 会话加物,B 会话看不到(跨会话不共享),故"换会话避锁"一法不通。

## 未决面

- 目标窗口=「支付校验通过」到「订单落定」之间塞入夹克(题面/race topic 明示的 add-during-window)。
- 已试齐发(件 `race_send` 升级 dual-endpoint):同时、add 延后 1/5/15ms、以及浏览器(Chrome,同 tick 双 `fetch`,HTTP/2)——
  订单始终只见便宜件(Total $10),夹克未被计入;即 add 总落在订单落定**之后**。add 提前则支付校验看到夹克 → 303。
- 结论:该窗口实际上在**单请求内**、亚毫秒级,需要真正等价的 single-packet 齐发(HTTP/2 同包)才能对齐;HTTP/1.1 双连接与 JS 同 tick
  fetch 均未命中。下一步候选:造 h2 同包件,或按 topic「abusing rate/resource limits」先制造服务端延迟再齐发。

## 证据摘录

```
(便宜车=$10) checkout + add(jacket) 同发/延后:  checkout -> 200 订单确认 Total $10(无夹克)
(夹克车)     checkout + remove(jacket) 同发:   checkout -> 303 INSUFFICIENT_FUNDS(先校验看到夹克)
Chrome 同 tick 双 fetch: add -> 200(跟到 /cart 空), checkout -> 200 Total $10
solved_check / -> {"solved":false}
```

## 复现命令

```
lab_page "https://portswigger.net/web-security/race-conditions/lab-race-conditions-multi-endpoint" --out /tmp/b6-4.html
lab_launch launch 167394BD4478DD05F683AFFD8D0BF579E9DAF428CB0808509BA8F76B1A12C046 --widget-source /web-security/race-conditions/lab-race-conditions-multi-endpoint --jar /tmp/mar-jar.json
lab_http post "<inst>/cart" --form productId=2 --form quantity=1 --form redir=CART --jar /tmp/mar-jar.json   # 便宜车
race_send "<inst>/cart/checkout" --url2 "<inst>/cart" --body "csrf=<csrf>" --body2 "productId=1&redir=CART&quantity=1" --n 1 --stagger-ms 1 --out-dir /tmp --jar /tmp/mar-jar.json
solved_check "<inst>/" --jar /tmp/mar-jar.json
```

新件/升级:`race_send` 增加 `--url2`(异构端点齐发)、`--stagger-ms`(变体 B 延时)、`--out-dir`(存各变体响应体)。
