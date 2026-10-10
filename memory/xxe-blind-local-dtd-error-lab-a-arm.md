---
metadata:
  node_type: memory
name: "XXE blind local-DTD error lab A arm"
description: "A-arm solve: blind XXE via repurposed local DTD (/usr/share/yelp/dtd/docbookx.dtd ISOamso) leaks /etc/passwd through parser error"
last_updated: 2026-10-10T01:46:03+08:00
created: 2026-10-10T01:46:03+08:00
---

Lab: /web-security/xxe/blind/lab-xxe-trigger-error-message-by-repurposing-local-dtd (widget id 9179F60D7EFF1096E159E290D8F14FACB04543CCC4380AEC40DE3F603DBD8364).

Path: range_launch launch-url -> instance; stock check at POST /product/stock (Content-Type: application/xml).

Payload shape: outer DOCTYPE declares %local_dtd = file:///usr/share/yelp/dtd/docbookx.dtd, then redefines the existing entity ISOamso with a nested chain:
- %file = file:///etc/passwd
- %eval defines %error = SYSTEM 'file:///nonexistent/%file;'
- invoke %eval; then %error; then %local_dtd;

Result: parser raises FileNotFoundException whose message embeds the /etc/passwd body -> HTTP 400 with the file contents in the JSON error field. banner_verdict then reports solved true.

Key point: the error channel is the read channel when the app hides the parse result; no external DTD hosting needed because a system DTD already exists on the target.

