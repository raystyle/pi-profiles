---
metadata:
  node_type: memory
name: "Lab GraphQL Brute-Force Protection Bypass"
description: "arm A solved: GraphQL alias packing bypassed the login rate limiter; carlos/robert via gql_alias_brute on /graphql/v1 (probe lesson: /graphql 404s)."
last_updated: 2026-10-10T10:59:36+08:00
created: 2026-10-10T10:59:36+08:00
---

## 2026-10-10 — lab-graphql-brute-force-protection-bypass (arm A)

Instance: https://0a4500fa0397b9388169a38300290079.web-security-academy.net/ (range_launch launch-url, reused:false, jar /tmp/cj1.json).

Path: page_read 题面 -> widget-lab-id 1ff12f... -> range_launch launch-url -> gql_alias_brute -> banner_verdict.

Method (alias packing): one POST carrying N aliased login mutations
`mutation{ a0:login(input:{username:"carlos",password:"..."}){token success} ... }`
so the whole password list is consumed inside a single request and the per-origin request-rate limiter never fires. 99 aliases in one call, status 200, winner located by the alias whose `success` flips true.

Findings:
- Endpoint discovery mattered: POST /graphql returned 404 "Not Found" (plain text). The real path is /graphql/v1, read from /resources/js/gqlUtil.js (`fetch('/graphql/v1')`), which the login page loads via /resources/js/loginGql.js. Mutation shape `mutation login($input: LoginInput!){login(input:$input){token success}}`, error text "Invalid username or password."
- Winner: alias a48, password robert, token rTtHBfCdhQ1ECAGQApD7ocLEBRoHbFRg; a token string comes back even for failures (all failures share one static token) so only `success` is the oracle.
- /my-account with that session cookie -> 200 account page; banner_verdict on base URL -> solved:true, "<h4>Congratulations, you solved the lab!</h4>".
- Wordlist source: /web-security/authentication/auth-lab-passwords; doc_read of that page loses the list (nav noise only), text_grep on the markdown recovers it as one backtick-wrapped space-separated line (line 105) - materialized to /tmp/pwlist.txt. gql_alias_brute counted 99 of 100 lines, so alias index -> password mapping must be read off the script's own list, not the file's visual order.

