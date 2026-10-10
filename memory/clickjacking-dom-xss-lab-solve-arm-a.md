---
metadata:
  node_type: memory
name: "Clickjacking DOM-XSS lab solve arm A"
description: "Solved lab-exploiting-to-trigger-dom-based-xss: the 64-char name limit forced a short print payload; button rect measured at iframe layout width 780 and the decoy aligned via a 4x-scaled iframe."
last_updated: 2026-10-10T21:07:48+08:00
created: 2026-10-10T21:07:48+08:00
---

Lab: /web-security/clickjacking/lab-exploiting-to-trigger-dom-based-xss (instance 0aa7002803ae7bb280e83fd90007006d, solved 2026-10-10).

Sink: /feedback loads submitFeedback.js. displayFeedbackMessage does `feedbackResult.innerHTML = "Thank you for submitting feedback, " + name + "!"` where name = FormData name field. The feedback form prefills every field from the URL query string client-side (verified: /feedback?name=MARKZQX&email=a@b.com fills input values). So deliver by clickjacking the Submit feedback button with the payload preset in the name query param.

Blocker found: the submit endpoint validates `Name must be length 64 or less.` -> long payloads (print plus a fetch beacon) are rejected and only the error text is injected. Working payload: `<img src=x onerror=print()>` (26 chars).

Geometry method that worked: iframe width 780 + height 1000 (page content height 927 -> no iframe scrollbar, so internal layout width is exactly 780), measure the Submit button rect in the harness browser at clientWidth 780 (hiding the parent scrollbar gives the same layout; rects at 765 and 780 were byte-identical here: button x16 y826.6 w184.2 h33). Then scale the iframe 4x: screen = left/top + 4*internal, anchored so the button centre maps onto the Click me decoy at 300,300 -> left:-132px, top:-3072px, transform-origin:0 0, transform:scale(4). The scaled button covers about 737x132 screen px, so the decoy click has slack.

Mapping proof (reusable): put the decoy over a link's mapped position (nav Home, internal centre 564.6,225), dispatch Input.dispatchMouseEvent press+release at the decoy centre, and detect the cross-origin iframe navigation via an iframe load listener flipping document.title. title=NAVOK proved the click-to-internal-coordinate mapping before delivering.

Delivery: exploit server POST formAction=STORE or DELIVER_TO_VICTIM with urlIsHttps=on, responseFile=/exploit (required; empty gives "File must start with /", absent gives "Missing parameter responseFile"), responseHead (2 lines, newline separator fine), responseBody; DELIVER must repeat head+body and needs --follow (302 -> /deliver-to-victim -> /). Read the log with GET /log.

XSS-channel side note: the img onerror handler execution was proven by overriding window.print to beacon and seeing GET /xss2 in the access log.

