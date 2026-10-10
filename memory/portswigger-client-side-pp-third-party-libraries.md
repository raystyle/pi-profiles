---
metadata:
  node_type: memory
name: "PortSwigger Client-Side PP Third-Party Libraries"
description: "Lab solved: fragment __proto__[hitCallback] PP via jQuery BBQ source + ga.js gadget, delivered from exploit server"
last_updated: 2026-10-09T19:54:25+08:00
created: 2026-10-09T19:54:25+08:00
---

Target: /web-security/prototype-pollution/client-side/lab-prototype-pollution-client-side-prototype-pollution-in-third-party-libraries

Instance root scripts (first-party paths, third-party code):
- /resources/js/ga.js            = Google Analytics ga.js -> GADGET (hitCallback)
- /resources/js/jquery_ba_bbq.js = jQuery BBQ           -> SOURCE (fragment deparam)
- /resources/js/store.js, jquery_1-7-1.js

Payload (one line):
  https://<lab>/#__proto__[hitCallback]=alert(document.cookie)

Evidence:
- page_alert on that fragment URL -> fired=true; alert text = __utma/__utmc/__utmz/__utmb GA cookie string (document.cookie).
- Exploit server STORE + DELIVER_TO_VICTIM of:
  script tag: location = "https://<lab>/#__proto__[hitCallback]=alert(document.cookie)";
- banner_verdict -> solved:true, congrats line "<h4>Congratulations, you solved the lab!</h4>"

Method notes (reusable):
- Fragment (#) is the source, not the query string: jQuery BBQ parses location.hash recursively, so pollution lands before ga.js reads its config.
- ga.js reads the polluted Object.prototype.hitCallback and calls it -> function-call gadget with attacker-controlled body.
- ga.js is injected by an inline snippet (createElement + insertBefore), so a plain "script src" grep of the root HTML misses it; grep for ga.src / _gaq instead.
- Exploit-server delivery: responseHead + responseBody must be repeated on the DELIVER_TO_VICTIM POST (responseFile alone 400s "Missing parameter responseFile"); the ACCESS_LOG POST also needs responseFile or it 400s.

