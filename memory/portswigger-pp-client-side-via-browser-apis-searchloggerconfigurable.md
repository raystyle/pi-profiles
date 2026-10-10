---
metadata:
  node_type: memory
name: "PortSwigger PP client-side via browser APIs - searchLoggerConfigurable"
description: "Solved the browser-APIs PP lab by polluting Object.prototype.value so defineProperty's descriptor inherits the payload; URL ?__proto__[value]=data:,alert(1);"
last_updated: 2026-10-10T21:18:19+08:00
created: 2026-10-10T21:18:19+08:00
---

Lab: /web-security/prototype-pollution/client-side/browser-apis/lab-prototype-pollution-client-side-prototype-pollution-via-browser-apis
Instance: 0a24001f042f306080ca5891008f00a0.web-security-academy.net (fresh, reused:false)

Surface read
- Home page (and /logout) load /resources/js/deparam.js + /resources/js/searchLoggerConfigurable.js; other pages load labHeader.js only, /feedback adds submitFeedback.js (no PP sink).
- Sink: `if(config.transport_url){ script.src = config.transport_url; document.body.appendChild(script) }`.
- Source: `deparam(new URL(location).searchParams.toString())` -> `?__proto__[k]=v` writes Object.prototype (verified: after navigating with the payload, Object.prototype.transport_url held the string).

Patch and why the naive payload fails
- `let config = {params: ..., transport_url: false}` followed by
  `Object.defineProperty(config,'transport_url',{configurable:false,writable:false})`
  makes transport_url an own non-writable data property (value false), so prototype pollution never reaches the read. Naive `?__proto__[transport_url]=data:,alert(1);` -> fired:false, DOM had no data: script.

Bypass (worked)
- ToPropertyDescriptor uses HasProperty/Get, which walk the prototype chain. The patch's own descriptor literal has no own `value`, so polluting Object.prototype.value makes the descriptor inherit [[Value]]; defineProperty then writes that value into the shielded own property.
- Payload: `/?__proto__[value]=data:,alert(1);` -> page_alert fired:true (alerts:["alert:1"]).
- Note: polluting `get`/`set` instead makes ToPropertyDescriptor throw (invalid descriptor, value+get), aborting searchLogger.

Verdict
- banner_verdict on the base URL after the alert: solved:true, congrats line present. No victim delivery needed on this instance.

Reuse
- For any defineProperty-shield gadget: try Object.prototype.value pollution before concluding the sink is dead.
