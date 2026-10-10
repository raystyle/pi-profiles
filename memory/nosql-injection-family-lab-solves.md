---
metadata:
  node_type: memory
name: "NoSQL injection family - lab solves"
description: "NoSQL injection detection lab solved via '||'1'=='1 category filter + banner verdict pattern"
last_updated: 2026-10-10T11:41:58+08:00
created: 2026-10-10T11:41:58+08:00
---

- 2026-06-XX lab-nosql-injection-detection (arm A, instance 0a2500d3...): product category filter `?category=` concatenated into a MongoDB query. Baseline `category=Gifts` returned 3 products; payload `Gifts'||'1'=='1` (URL-encoded `Gifts%27%7C%7C%271%27%3D%3D%271`) returned 16 products (unreleased ones) -> solve.
- Method: detect with a single quote for the server error, then close-and-OR the string comparison so the `$where`/match becomes always-true and the whole collection returns.
- Verification: banner_verdict on the instance root flipped to is-solved with the congrats line. The banner inside the very injection response still read "Not solved" - the solve check lands on the next page read, so re-read the banner after the payload.
- Tooling used: page_read (lab page -> lab id), range_launch launch-url, http_session get with --jar, banner_verdict.

