---
metadata:
  node_type: memory
name: "PortSwigger clickjacking lab victim bot"
description: "PS clickjacking lab bot clicks the \"Click me\" decoy centre; how to instrument it and find the button band"
last_updated: 2026-10-11T01:40:01+08:00
created: 2026-10-11T01:40:01+08:00
---

- Victim bot click model (lab-prefilled-form-input, verified by instrumentation): the bot dispatches a REAL mouse click at the centre of the "Click me" decoy element in the top document. A same-origin self-framing probe (outer page frames `/exploit?inner=1` of the same file) logged `down x=108 y=184 target=btn` and the parent's `window.blur` at the same instant, so the click lands on the decoy's bounding-rect centre and is routed into the iframe (cross-origin frames get it too).
- Load/click timing is NOT the failure mode: the framed lab page fired `frame-load` at t=121 ms (cached) and the click/blur came 0.4-3.5 s after `parent-load`.
- Readable diagnostic channel: `new Image().src='/hitlog?...'` from the exploit page lands in the exploit server access log (`GET /log`) with full query string; exploit server answers 404/whatever, the line is what matters.
- Useful instruments: iframe `load` counter (a SECOND frame load = the click submitted a form inside the frame), parent `blur` (click went into the frame), parent `mousedown` coords (click stayed in the parent).
- Exploit-server constraint: it stores ONE file - storing a second `responseFile` 404s the first. Self-framing (`src="/exploit?inner=1"`) gives a same-origin child without a second file.
- Position-finding method that solved it: sweep the decoy's `top` across the bot's reloads (localStorage counter, 15 positions step ~30px), keep the decoy 33 px tall so a centre click covers the whole button, and read which band yields `frameload2`. Fixed hit band here: decoy top 435-485 (button frame-local rect left 32, top ~467 in a 780x700 frame).
- Pitfall (self-inflicted): resetting the account email via my own POST to /my-account/change-email raced the victim's click and clobbered the changed state; only read, never re-POST, lab account state while a delivery loop is running.
- Working payload shape: 780x700 iframe (opacity 0.0001, z-index 2) framing /my-account?email=... , decoy div at the button's frame-local rect (z-index 1, so the click hits the frame), body margin 0.
