---
title: AngularJS 1.4.4 sandbox escape practice notes
---

# AngularJS 1.4.4 sandbox escape practice notes


Family: [[client-side-template-injection-family]]. Measured on the Academy CSTI labs (instances of batch 38) with `browser_suite eval` against the page's own injector: `angular.element(document.body).injector().get('$parse')`.

## The two escapes you actually need

1. **charAt override** (defeats the rewriting): `toString().constructor.prototype.charAt=[].join`. After it runs, `String.prototype.charAt` IS `[].join`, `isIdent()` always returns true, and the sandbox's AST→code rewriting stops inserting its guards for that parse. Verified: `String.prototype.charAt` becomes `function join() { [native code] }`.
   - It only affects **later** parses. Your payload must therefore be parsed *after* the override — e.g. a second query parameter treated as an expression, or a value that Angular `$parse`s at runtime (the `orderBy` filter's string predicate).
2. **CSP variant**: with `ng-csp` the Function constructor is avoided entirely; the working route is Angular's own events plus `$event`: `$event.composedPath()` returns an array (it passes the expensive checks, unlike `$event.view`), and `|orderBy:'<expr>'` evaluates `<expr>` once per element with that element as the scope — the path's last element is `window`, so `(y=alert)(document.cookie)` (assignment-then-call) reaches a global function without tripping the window/function guards.

## Traps measured this batch

- **`event.path` is gone in current Chrome** (`(new Event('focus')).path === undefined`). Only `composedPath()` (21 chars) works.
- **`ng-focus` + a URL fragment focus does not fire on a fresh load**: the fragment focus happens during/at the end of parsing, before Angular bootstraps and attaches the directive listener. Diagnostic: `document.activeElement.id` already equals the target id, yet no handler effect. `autofocus` flushes at *load* (after bootstrap) and does work, but costs 10 extra characters — decisive when the lab has an 80-character limit (79-char fragment payload + `autofocus` = 84 > 80 → dead end as-is).
- **Reassigning `iframe.src` to the same URL plus a hash does not help** for an iframe you cannot reach into (cross-origin); it either reloads or the focus still precedes the child's bootstrap.
- **Parameter order is not query order**: an app that emits one `$parse` block per query parameter iterates them in **name order (alphabetical)**, so a "override first, payload second" plan needs the payload's parameter *name* to sort after the override's (e.g. prefix the payload expression with `zzz;`).
- **After the override, the lexer breaks on some payloads**: `$parse("zzz;alert(1)")` → `[$parse:lexerr] Unexpected next character [z]`, and `$parse("zzz;constructor.constructor(97,108,...)()")` likewise, while `zzz;constructor.constructor(toString().constructor.fromCharCode(97,...))()` lexes — then dies on `[$parse:isecobj]`. So `constructor.constructor` still hits `ensureSafeObject`; the override defeats isIdent, not the runtime guards. Plan the execution primitive accordingly (prefer the orderBy/scope-element route over the direct Function route).
- Character sources without quotes: `(1+[]).constructor` and `toString().constructor` are `String`; `[].filter.toString()` gives text to slice; `[]+{}` is `"[object Object]"`. `constructor`, `toString.constructor`, `[].filter.constructor`, `.call/.apply/.bind` are all blocked directly (isecobj/isecfn/isecff).
