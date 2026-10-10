---
metadata:
  node_type: memory
name: "Lab Race Conditions Multi Endpoint"
description: "Solved PortSwigger multi-endpoint race lab: single-packet h2 burst of one jacket add + three checkouts against a $10 cart"
last_updated: 2026-10-10T09:58:57+08:00
created: 2026-10-10T09:58:57+08:00
---

## 2026-02-13 lab-race-conditions-multi-endpoint (arm A, solved)

Target: PortSwigger academy shop, jacket $1337 (productId=1), gift card $10 (productId=2), credit $100.

Mechanic (empirically established): `POST /cart` (add) and `POST /cart/checkout` are two endpoints whose
session read-modify-write interleaves. Checkout validates the cart total and builds the order across that
window; an add landing inside the window puts the expensive item in the order priced off the cheap cart.

Winning shape (one h2 single-packet burst, `h2_burst`):
1. Sequentially add one gift card -> cart total $10 (affordable, so a checkout can actually validate).
2. ONE packet: `POST /cart productId=1&redir=PRODUCT&quantity=1` + THREE `POST /cart/checkout` with the
   session csrf. burst_bytes 741 (< MSS), statuses [302,200,200,200].
3. The checkout responses themselves already carried `academyLabBanner is-solved`; banner_verdict on `/`
   confirmed `solved:true` + "Congratulations, you solved the lab!".

Round-1 miss (recorded because it is the instructive case): 1 add + 1 checkout did NOT land. Result was
credit 100->90, cart empty, and a gift card code (AcpPhAGYZ3) on /my-account: the checkout had read the
pre-add cart (gift card only) while the add's session write was clobbered by the checkout's cart clear.
So a single concurrent pair is a coin flip; widening the checkout side is what converts it.

Piece notes: `h2_burst` does NOT add a Content-Type header - POST bodies need the third pipe field
(`|Content-Type: application/x-www-form-urlencoded`) or the form is not parsed. csrf is per-session
constant (same token on /cart, /my-account forms). --warm 1 keeps TLS/H2 warm; --jar supplies the session
cookie. Retry path per the lab hint: a placed gift-card order yields a code redeemable via
`POST /gift-card` (csrf + gift-card=CODE) to top the credit back up, so failed rounds are self-funding.
Discipline respected: no solution/records/family material read; all HTTP through pieces.

