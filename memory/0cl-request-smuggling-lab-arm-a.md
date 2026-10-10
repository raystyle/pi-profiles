---
metadata:
  node_type: memory
name: "0CL request smuggling lab arm A"
description: "PortSwigger 0.CL lab: Expect-based front-end/back-end CL desync confirmed and smuggled XSS request executes; victim delivery via RQP not landed"
last_updated: 2026-10-09T17:24:21+08:00
created: 2026-10-09T17:24:21+08:00
---

Lab: PortSwigger `/web-security/request-smuggling/advanced/lab-request-smuggling-0cl-request-smuggling`
(goal: alert() in Carlos's browser; Carlos polls `/` every 5 s). Instance host is an HTTP/2-capable
front-end that also serves HTTP/1.1 and proxies to an HTTP/1.1 back-end.

Confirmed 0.CL primitive (front-end blind to the body, back-end sighted):
- `Expect: 100-continue` breaks the front-end's body handling: with
  `GET / HTTP/1.1 + Host + Expect: 100-continue + Content-Length: L`, the front-end forwards only
  the header block to the back-end (it "forgets" it must still receive a body from the client),
  while the back-end honours the Content-Length and consumes L bytes of whatever follows.
- Working POC: primer `GET / ... Expect: 100-continue + Content-Length: <carrier-header-block-len>`
  followed by a carrier `GET / HTTP/1.1 + Host + Content-Length: M + CRLF + body`. The back-end eats
  the carrier's header block as the primer's body, then parses the carrier's body as fresh requests,
  so a hidden request executes. Proven: hidden `GET /nope12345` returned an extra 404 in the stream,
  and a hidden `GET /post?postId=1` returned the ~7.9 KB XSS page.
- Carrier header block length must be exact (front-end does NOT append headers here). Host is 57
  chars: carrier header = 16 + "Host: "+57+2 + ("Content-Length: M"+2) + 2 = 104 for a 3-digit M.
- Second blind form: `Content-Length : N` (space before colon) - the front-end forwards only the
  header block and the back-end waits, i.e. the plain 0.CL deadlock. desync_probe "0 responses" rows
  were this deadlock signature, not rejects.

XSS vector: `/post?postId=N` reflects the request `User-Agent` UNESCAPED into a hidden form field:
`<input ... name="userAgent" value=""><svg/onload=alert(1)>">` - UA payload `"><svg/onload=alert(1)>`.

Blocker (unsolved): delivering the smuggled XSS response to the victim. The front-end relays every
back-end response (incl. the extra one) to the arming downstream connection, so the smuggle response
reaches the attacker, not Carlos. Drop-fast spraying (arm then close before the response lands, so
the extra stays pending) did not flip the banner over ~6 min; cross-connection probes of `/` always
got the normal homepage. Next angles: determine whether the front-end's upstream connection is truly
per-downstream (if so, victim delivery needs the 0.CL to CL.0 double-desync leaving a malicious
prefix the victim's own request completes), or pin the extra response via the front-end's
static-asset early-response/cache path.

Tooling added: project pieces `h2seq` (same-connection H2 arm+follow sequencer) and `zerocl_spray`
(Expect-0.CL RQP spray) under `.pi-rs/rust-scripts/`.

