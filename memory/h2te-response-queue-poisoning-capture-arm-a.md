---
metadata:
  node_type: memory
name: "H2.TE response-queue-poisoning capture arm A"
description: "H2.TE response-queue poisoning: TE header plus empty-chunk body desync, sustain-mode capture of the admin bot's login Set-Cookie, then /admin/delete; h2_te_steal piece authored"
last_updated: 2026-10-11T00:03:27+08:00
created: 2026-10-11T00:03:27+08:00
---

Lab: response-queue-poisoning via H2.TE (portswigger advanced request-smuggling). Instance 0adf006c03930bc0807d805400a50002.

Mechanism confirmed empirically: an HTTP/2 request carrying a plain `transfer-encoding: chunked` header plus a DATA body `0\r\n\r\n<smuggled H1 request>\r\nHost: <host>\r\n\r\n` is downgraded verbatim; the back-end ends the chunked body at the empty chunk and parses the tail as a second request, leaving one extra response in the front-end's upstream queue. Proof: 3 armed requests on one h2 connection, and the third stream received the 404 for the smuggled `GET /nope123zz`.

Capture geometry that worked: sustain mode - one long-lived h2 client connection (pins the upstream socket) plus paced probes, arm_every=2 so the queue never drains (a request that reads a queued response leaves its own behind, so the leftover count is self-sustaining). The admin bot's own login response then surfaced on one of our streams: foreign hit `302 location=/my-account?id=administrator` carrying `Set-Cookie: session=UHRG3prXFxpRPkMyW5rMkvKBtNCjKeOl`, and that session returned 200 on /admin.

Numbers: 470 requests / 60 rounds / 105 s, 137 captured session candidates, 10 reconnects (the front-end closes the client connection when driven hard - reconnect and continue rather than abort), 329 reads with no response (stream/queue loss is normal here).

Objective close: the panel's delete link is a plain GET `/admin/delete?username=carlos` (no csrf) - 302 to /admin, `User deleted successfully!`, and the banner flipped to is-solved in the same response.

Tooling: authored `.pi-rs/rust-scripts/h2_te_steal.rs` (project tier, hpack-based H2 framing, TE-armed sender + sustained prober + foreign-response log + session dedup/verify + optional --delete-user).

Lesson: for RQP labs, do not re-arm per round from fresh connections and do not abort on connection close; keep exactly one leftover pending on a pinned upstream socket and let the victim's response rotate into your probe queue.
