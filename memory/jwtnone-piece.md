---
metadata:
  node_type: memory
name: "jwt_none piece"
description: "New project piece jwt_none forges unsigned alg:none JWTs plus variant jars; solved the lab-jwt-authentication-bypass-via-flawed-signature-verification instance with it."
last_updated: 2026-10-10T12:05:44+08:00
created: 2026-10-10T12:05:44+08:00
---

## 2026-10-10 - jwt_none piece + lab-jwt-authentication-bypass-via-flawed-signature-verification (arm A)

Target: /web-security/jwt/lab-jwt-authentication-bypass-via-flawed-signature-verification. Solved.

Shape of the solve:
- `page_read` on the canonical lab path yields the widget lab_id directly (no /api/widgets fallback needed); `range_launch launch-url <path> --jar /tmp/cj1.json` gave a fresh instance (reused:false).
- PortSwigger login needs the `csrf` hidden field; `http_session submitform` reported "Missing parameter" while a plain `post --form csrf=... --form username=wiener --form password=peter` worked (that csrf value is not bound to a session cookie - the GET only set `session=;` empty).
- Banner read "Not solved" before; after the delete action banner_verdict returned solved:true with the congrats line.

Gap found and closed: the bundled `jwt` piece covers decode / JWKS-to-PEM / HS256 algorithm-confusion but has NO `alg:none` path, so unsigned-token work had no piece. Authored project piece `.pi-rs/rust-scripts/jwt_none.rs` (v1.0.0, project tier - discovered by rs_search, no bundled catalog regen needed, only one copy exists since .pi-rs is outside the searchable tree).

jwt_none contract: `<token> [--sub S] [--kid K] [--algs 'none,None,nOnE'] [--host H] [--cookie session] [--jar-out DIR] [--out FILE]`. It decodes the captured JWT, rewrites claims, and emits the cross product of alg spelling x typ present/absent x trailing-dot present/absent (12 variants), each with an empty signature, plus one http_session-format jar per variant (`<DIR>/jar-<alg>[-notyp][-nodot].json`, host -> cookie -> token) so the next http_session call presents it. Jar format confirmed: `{"<host>":{"<name>":"<value>"}}`, host without port.

Evidence: forged `{"alg":"none","kid":...,"typ":"JWT"}` + `{"sub":"administrator"}` with empty sig presented as the `session` cookie returned 200 on /admin with "My account?id=administrator"; `/admin/delete?username=carlos` returned 302 to /admin; banner then reported solved:true.

Trap worth remembering: the first version slugged the variant name with `alg.to_lowercase()`, so none/None/nOnE all wrote the same jar path and the last write won - a silent collision that made the variant matrix useless. Case-folding is never safe in a variant filename; strip non-alphanumerics only, and verify distinct paths in the run output before trusting the matrix.

