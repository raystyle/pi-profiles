---
title: "lab-logic-flaws-infinite-money"
links:
  - target: logic-flaws-family
    relation: evidences
---

# lab-logic-flaws-infinite-money

> evidences: [[logic-flaws-family]]

神秘题(第 29 批,2026-10-06):`/web-security/mystery-lab-challenge` 抽到的实例
`https://0aba00fd032862bc8216f6e80090001c.web-security-academy.net/`。
判定:**solved**(banner `is-solved` + `Congratulations, you solved the lab!`)。

## 定题(神秘题面是隐藏的,但**页面自己在 base64 里带着**)

根页有 `<a id=mysteryReveal data-hidden-objective='<b64>' data-hidden-link='<b64>'>`:

- objective → `Purchase a "Lightweight l33t leather jacket" from the online store.`
- link → `logic-flaws/examples/lab-logic-flaws-infinite-money`

分类面:shop + 20 个商品 + `/cart` + `/sign-up`(仅 newsletter)+ `/login` + `/my-account` +
`/gift-card` 兑换 + `submitSolution`;`wiener:peter` 可登录(货架 lab 的固定凭据)。

## 逻辑缺陷(无限刷钱)

- 礼品卡 `productId=2` 单价 $10;优惠券 **`SIGNUP30`**(30% off,`POST /cart/coupon`);
  **折扣会作用在礼品卡上**,而卡可以**按面值**兑换 → 每轮净赚 30%。
- 一轮:`POST /cart {productId:2,redir:CART,quantity:q}` → `POST /cart/coupon {csrf,coupon}` →
  `POST /cart/checkout {csrf}`(303 → `/cart/order-confirmation?order-confirmed=true`)→
  确认页 `<p>You have bought the following gift cards:</p>` 下每行一个 10 位码 →
  `POST /gift-card {csrf,gift-card:<码>}`(用 `/my-account` 页的 csrf)。
- 取 `q = floor(credit / (price*0.7))`(卡面价 × 折扣后 ≤ 余额);余额 100 → 5 → 55 → 76 → … 指数增长,
  8~12 轮即可超过 $1337。

## 工具:`money_loop`(新件,可复用)

`money_loop <base-url> <user> <pass> [--coupon SIGNUP30] [--gift-product 2] [--price 10] [--target 950] [--max-qty 99] [--cycles 16] [--jacket-product 1]`
—— 登录 → 循环(加购/用券/结账/取码/逐张兑换)→ 买 jacket → 报告 credit 曲线、兑换数、终态。
实测:credit 503.1 → 1013.1,jacket `due 935.90` → `checkout=303`,终credit 77.2。耗时约 7 分钟(run 在后台跑)。

## 踩坑(重要)

- **判定翻牌要"渲染确认页"**:只 `POST /cart/checkout` 拿 303 不翻牌;必须紧接着
  `GET /cart/order-confirmation?order-confirmed=true` 才会把 lab 标 solved(第一轮漏了这步 → 订单成功但 banner 未翻)。
- 同一 session 才能看订购单确认页;脚本内保持同一 cookie jar。
- 偶发 `tls connection reset by peer`(连发太快)→ 脚本内 4 次重试 + 兑换间隔 60ms 即可。
