---
metadata:
  node_type: memory
name: "PRS arm A lab-javascript-string-single-quote-backslash-escaped"
description: "arm A 基线:lab-javascript-string-single-quote-backslash-escaped 冷实例一次通过 - </script><script>alert(1)</script> 破串,page_alert fired + banner solved"
last_updated: 2026-10-09T00:09:50+08:00
created: 2026-10-09T00:09:50+08:00
---

Arm A baseline: lab-javascript-string-single-quote-backslash-escaped solved in one pass.

- page_read canonical path -> lab_id D926F2EB...F97442 (no 404, slug path valid).
- range_launch --jar /tmp/cj1.json -> reused:false, instance https://0add00ec039c37c880d2031800b900e9.web-security-academy.net/.
- Recon: GET /?search=test123, http_dump (unstripped body) line 60 `var searchTerms = 'test123';`.
  Context = search term reflected inside a `<script>` block JS string; `'` and `\` escaped, angle brackets NOT encoded.
- Payload: `</script><script>alert(1)</script>` (no quotes/backslashes needed -> escaping irrelevant).
  Reflected verbatim at line 92; HTML parser closes the script element at `</script>` and runs the injected block.
- Evidence: page_alert fired=true alerts=["alert:1"]; banner_verdict solved:true, "Congratulations, you solved the lab!".
- Traps: http_session strips <script> blocks (its saved body hides the JS reflection -> empty slot is NOT non-reflection);
  use http_dump --out for the raw script line. Reflected-XSS labs here need no exploit-server victim delivery.
- Pieces used: page_read, range_launch, http_dump, text_grep, http_session, page_alert, banner_verdict. No new piece needed.

