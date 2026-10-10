---
metadata:
  node_type: memory
name: "rs pieces authored"
description: "New piece coupon_cycle (alternating-coupon business-rule loop) authored and proven on the flawed-enforcement lab."
last_updated: 2026-10-10T02:54:00+08:00
created: 2026-10-10T02:54:00+08:00
---

- 2025 lab session (logic-flaws/coupon alternation): authored project piece `coupon_cycle` (.pi-rs/rust-scripts/coupon_cycle.rs) - logs in, fills the cart with a target product, applies two coupon codes round-robin while the app blocks only the immediately previous code, stops at cart total <= store credit, checks out and reports the price curve, accept/reject sequence, checkout receipt and solved banner. First-envelope run on the flawed-enforcement lab: 7 applications, total 1337 -> 0.00, solved.
