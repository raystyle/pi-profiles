---
metadata:
  node_type: memory
name: "PortSwigger TE.CL front-end control bypass"
description: "TE.CL desync solved on a closing front-end via side-effect oracle; winning 205-byte frame and front-end parsing findings"
last_updated: 2026-10-09T15:04:02+08:00
created: 2026-10-09T15:04:02+08:00
---

Instance: https://0abf008903241ce681249ee500e60001.web-security-academy.net/ (launched via range_launch launch-url on the canonical path). Goal: smuggle a request to the back-end that reaches /admin and deletes carlos. External /admin = 403 `"Path /admin is blocked"`.

Front-end/back-end reading (conn_reuse only):
- TE alone with an incomplete chunk (`28\r\n`, no data) hangs -> the front-end really parses chunked.
- TE + CL with a complete short chunked body hangs -> the wait is at the back-end (CL honoured there). TE.CL confirmed; the lab statement also says the back-end has no chunked support.
- Header order (CL first vs TE first) changes nothing.
- A frame whose chunk body holds a whole `GET /admin ...` request returns 200, not 403 -> the front-end consumes those bytes as the chunked body (no path check on them).

Delivery limit: the front-end closes the client connection after ONE response even with `Connection: keep-alive`; two pipelined GETs in one write yield a single response. So the smuggled response is unreadable and the poisoned back-end connection is not reused: arm + 4 fresh-connection probes stayed clean (200 / 8458 bytes).

Working oracle: the side effect, not the response. The back-end executes the smuggled request even though its response never reaches the client. Judge the desync by target state (banner_verdict), never by the second response.

Winning frame (205 bytes, sent once): outer `POST /` with `Content-Length: 4` + `Transfer-Encoding: chunked`, then chunk size `3f`, inner request `GET /admin/delete?username=carlos HTTP/1.1\r\nHost: localhost\r\n\r\n` (63 bytes), then `\r\n0\r\n\r\n`.

Evidence: banner_verdict -> solved_class true, `<h4>Congratulations, you solved the lab!</h4>`.

Lessons: (1) the hex chunk size must count the inner request exactly - one byte off yields 400 `{"error":"Invalid request"}` from the front-end; (2) a closing front-end does not mean the chain is dead - switch to side-effect judging; (3) outer CL = hex-len + 2 (`3f` -> 4) so the back-end stops right after the chunk-size line.

