---
metadata:
  node_type: memory
name: "Lab Solve: Race-Condition Rate Limit Bypass"
description: "Rate-limit race lab solved: last-byte-sync h2 burst (new h2_lbs_race piece) plus case-variant usernames as fresh rate-limit keys; carlos=1234567."
last_updated: 2026-10-10T08:45:45+08:00
created: 2026-10-10T08:45:45+08:00
---

Target: PortSwigger race-conditions/lab-race-conditions-bypassing-rate-limits (instance 0a660084...web-security-academy.net). Solved: brute-forced carlos (password 1234567), logged in, deleted user carlos via /admin/delete; banner_verdict solved:true, "Congratulations, you solved the lab!".

What worked (in order):
1. page_read gave the widget lab id; range_launch launch-url started the instance; the page's candidate list is exactly 30 passwords (123123 ... 000000).
2. h2_burst (one-write burst, 6073 B) was NOT simultaneous enough: 30 requests spread over several packets, only ~4 slipped past the lock.
3. Wrote new piece h2_lbs_race (.pi-rs/rust-scripts/h2_lbs_race.rs): last-byte-sync. Arms HEADERS + body-minus-last-byte with no END_STREAM for every stream, then releases every stream's final byte in ONE write (10 B/stream, END_STREAM) so all requests complete inside one packet no matter how large the burst is. 30 streams -> release_bytes 300.
4. Defect found and fixed in that piece: the reader never sent WINDOW_UPDATE, so the 65535 B h2 connection window stalled after ~13 response bodies. Sending WINDOW_UPDATE for a finished stream is a protocol error (server answers GOAWAY, received_bytes 17). Fix: send SETTINGS initial-window-size 2^31-1 plus one connection-level WINDOW_UPDATE up front (v1.0.2/1.0.3).

Rate limiter behaviour measured on this lab:
- Lock message "You have made too many incorrect login attempts. Please try again in N seconds." (N escalates 60 -> 120), page 4077/4078 B; invalid page 4026 B.
- A locked account masks even the CORRECT password (sequential test: wrong1 invalid, then all locked including the right one).
- Counter resets after the stated window; a fresh session is irrelevant (lock is not session-scoped).
- 30 requests also tripped a platform-level block: 3600 B page "Want to try again? You have failed - the account won't be unlocked before the timer runs out." served to any path and to cookie-less clients (IP-wide); it survived >90 s and only a lab restart cleared it.
- Per burst, only the first ~7-8 candidates are evaluated before the lock engages; the rest come back as lock pages.

The move that finished it: the login username match is CASE-INSENSITIVE (WIENER:peter -> 302 /my-account?id=wiener), while the rate limiter keys on the raw submitted username. So each casing of carlos is a fresh limiter key. One 15-request burst with a distinct casing per remaining candidate (carlos, Carlos, CARLOS, cArlos, caRlos, ...) evaluated all 15 at once; stream for caRlos+1234567 returned 302. Login with a still-fresh casing (CaRlos:1234567) because lowercase carlos was locked.

Reusable: h2_lbs_race <url> --req 'POST /path|k={pw}&..' --pw 'a,b,c' [--var k=v] [--jar J] [--arm-delay-ms N] [--read-ms N]; rows carry status/message/solved/invalid/rate_limited.

