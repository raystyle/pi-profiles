---
metadata:
  node_type: memory
name: "lab-logic-flaws-low-level A arm"
description: "Low-level logic flaw (cart 32-bit total wrap) solved with the new piece cart_int_overflow: quantity capped at 99 per request, wrap at request 163, land at $4.86, checkout solved."
last_updated: 2026-10-10T05:21:13+08:00
created: 2026-10-10T05:21:13+08:00
---

Target: PortSwigger "Low-level logic flaw" (/web-security/logic-flaws/examples/lab-logic-flaws-low-level), instance 0a1800900416f44d8028446d002a0065.web-security-academy.net, wiener:peter, credit $100.00, jar /tmp/cj1.json.

Flaw: the cart total is a signed 32-bit int. POST /cart validates quantity per request (100 and 20000 both -> 400 "Invalid parameter: quantity"), so the overflow is reachable only by accumulation of many <=99 adds - not by one big quantity.

Ruled out: quantity=-1 returns 302 but the line is dropped (cart stays empty); negative quantity only removes, it is not a usable flaw here.

Method (new piece cart_int_overflow, project tier, one envelope):
1. GET /cart for the start total; add 1 jacket if the jacket is absent.
2. Climb: POST /cart productId=<jacket>&quantity=99 (133700 cents per add) while the total is >= 0.
3. After the wrap (total < 0): greedily take the largest step min(99, |total|/price) that does not cross zero, over the product table sorted by price descending.
4. When the total falls in (-cheapest_price, 0], add one cheapest unit -> small positive total.
5. POST /cart/checkout with the csrf from /cart -> 303 to /cart/order-confirmation?order-confirmed=true.

Evidence (328 requests, prediction matched the page at every checkpoint):
- 60 -> $7,943,117.00; 120 -> $15,884,897.00
- 163 -> -$21,373,166.96 (wrap point)
- 180 -> -$19,122,995.96; 240 -> -$11,181,215.96; 300 -> -$3,239,435.96
- final total $4.86 (<= $100.00 credit), checkout 303, banner_verdict: solved_class true, "Congratulations, you solved the lab!".

Gotchas worth keeping: an empty cart page has no <th>Total:</th> row (treat "Your cart is empty" as 0); the app renders a negative total as "<th>-$21373166.96</th>" (sign BEFORE the $), so a naive "<th>$" needle mis-parses it; keep the raw totals row in the trajectory, the first live run aborted on exactly that parse.

Offline check: cart_int_overflow --selftest simulates the wrap + greedy (wrap at request 163, 329 requests, final 96 cents).
