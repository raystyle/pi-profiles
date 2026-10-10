---
metadata:
  node_type: memory
name: "Lab: DOM clobbering attributes to bypass HTML filters"
description: "Reused solved instance: banner is a stale pre-state (prior victim line in access log), and re-delivery does not re-summon the victim - no second victim-side execution possible"
last_updated: 2026-10-10T05:45:41+08:00
created: 2026-10-10T05:44:16+08:00
---

- Lab: /web-security/dom-based/dom-clobbering/lab-dom-clobbering-attributes-to-bypass-html-filters (PRACTITIONER, HTMLJanitor filter bypass), instance 0a5200d503f4535283b21e9000e900ec, solved.
- Filter location: comments are stored raw server-side and sanitized CLIENT-side at render time - /resources/js/loadCommentsWithHtmlJanitor.js builds `new HTMLJanitor({tags:{input:{name:true,type:true,value:true},form:{id:true},i:{},b:{},p:{}}})` and runs `janitor.clean(comment.body)` inside a `<p>`. So the browser (victim) is the filter execution point - not the POST.
- Bypass mechanism: htmlJanitor's attribute loop is `for (var a = 0; a < node.attributes.length; a++)`. A `<form>` containing `<input name=attributes>` makes `form.attributes` resolve to that named child element instead of the NamedNodeMap, so `.length` is undefined, `0 < undefined` is false, the loop body never runs and NO attribute is ever stripped from the form.
- Kept payload (allowed tags only, so nothing else is dropped): `<form id=x tabindex=0 onfocus=print()><input name=attributes>` posted to /post/comment as the comment body.
- Verified in-browser before delivery: rendered comment markup = `<p><form id="x" tabindex="0" onfocus="print()"><input name="attributes"></form></p>` - onfocus survived sanitization.
- Auto-execute: comments arrive via XHR AFTER page parse, so a bare `location='...#x'` cannot focus anything (element does not exist yet). Exploit server body: `<iframe src=".../post?postId=1" onload="setTimeout(()=>this.src=this.src+'#x',1000)"></iframe>` - the delayed fragment navigation runs the focusing steps on the focusable form -> onfocus -> print().
- Delivery: STORE then DELIVER_TO_VICTIM with urlIsHttps/responseFile/responseHead/responseBody all repeated (responseHead "HTTP/1.1 200 OK\nContent-Type: text/html; charset=utf-8") and --follow; the followed response itself carries `is-solved` + "Congratulations". banner_verdict confirmed solved=true on the instance root.
- Reusable rules: (1) locate the filter first (script tags are stripped from http_session-saved HTML - read them via browser eval or raw fetch) and clobber the exact property the sanitizer dereferences; (2) a client-side-only sanitizer means the POST channel is irrelevant - only the victim-side render matters; (3) when the injected node is created asynchronously, focus must be triggered by a delayed fragment navigation, never by the initial URL.


## 2026-10-09

- Reused-instance re-run observation (same lab, second arm-A pass): range_launch returned reused:true on the same host, and banner_verdict already reported solved=true BEFORE any attack. The prior run's victim line is in the exploit server access log (`10.0.3.83 ... "GET /exploit/" 200 "user-agent: Mozilla/5.0 (Victim) ... Chrome/154.0.0.0"`), so the banner was a stale verdict from the earlier solve, not a fresh transition - always read the banner first on reused:true and treat it as a pre-state, not as evidence.
- Re-delivering DELIVER_TO_VICTIM on an already-solved instance returns 302 -> /deliver-to-victim -> 302 GET / but NO new Victim fetch appears in the access log (checked immediately and again after a nap): once solved, the bot is not re-summoned, so a used-up instance cannot yield a second independent victim-side execution. The mechanism still re-attests mechanically there (fresh comment POST stored; render still yields `<form id="x" tabindex="0" onfocus="print()">`), but the only victim-side proof on such an instance is the historical log line.

