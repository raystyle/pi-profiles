---
metadata:
  node_type: memory
name: "Lab Solve WebSocket Handshake XSS"
description: "Solved PortSwigger WebSocket handshake lab: IP-ban bypass via X-Forwarded-For plus case-sensitive filter evasion (uppercase ONERROR, concatenated alert); new ws_msg piece."
last_updated: 2026-10-10T19:59:58+08:00
created: 2026-10-10T19:59:58+08:00
---

## PortSwigger lab-manipulating-handshake-to-exploit-vulnerabilities (arm A)

Target: live chat over WebSockets; described as "aggressive but flawed XSS filter".
Endpoint: `wss://<instance>/chat`. Protocol (from /resources/js/chat.js): client sends
`READY`, then `{"message":"..."}`; server frames are `{"user","content"}` / `{"error"}` / `TYPING`.

Chain that solved it (3 facts, each observed):
1. Benign `{"message":"hello"}` connects and echoes; `<img src=1 onerror=alert(1)>` returns
   `{"error":"Attack detected: Event handler"}` and closes -> the offending IP is banned.
2. While banned, the WebSocket handshake itself returns `HTTP 401 Unauthorized`. That is the
   flaw: the ban is keyed on the client IP and the handshake trusts `X-Forwarded-For`.
   Injecting `X-Forwarded-For: <any other ip>` on the handshake restores 101.
3. Content filter is a case-sensitive blocklist with separate reasons:
   - lowercase `on\w+=` -> "Attack detected: Event handler" (uppercase `ONERROR=` passes)
   - literal `alert` -> "Attack detected: Alert" (string-concatenated alert passes)
   Working payload: `<img src=1 ONERROR=window['al'+'ert'](1)>` delivered, echoed, agent
   browser fired alert -> banner "Congratulations, you solved the lab!".

Tooling: no bundled piece could inject handshake headers, so authored project piece
`ws_msg` (.pi-rs/rust-scripts/ws_msg.rs) - raw handshake via tungstenite with `--header` and
`--msg` repeats plus `--jar`.

