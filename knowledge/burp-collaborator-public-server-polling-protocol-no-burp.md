---
title: "Burp Collaborator public-server polling protocol (no Burp)"
---

# Burp Collaborator public-server polling protocol (no Burp)

# Burp Collaborator public-server polling protocol (no Burp)

## Question
Can a standalone Rust/Python client read interactions from the PUBLIC Burp Collaborator server (`oastify.com` / `burpcollaborator.net`) without Burp Suite? What is the exact subdomain-generation and polling protocol?

## Answer (synthesized)
YES — it is doable. The public server accepts a plain HTTP GET and returns JSON.

1. **Secret → subdomain.** Each client generates a 32-byte random secret; `biid` = base64 of those 32 bytes. The payload subdomain label is a *deterministic one-way derivation* of the secret (official: "derived from a one-way hash (cryptographic checksum) of the secret"). No registration step exists. Exact algorithm (from two agreeing implementations, unverified against PortSwigger source): `key_hash = first20( base36(SHA1(raw_biid) as 160-bit BE int) )`, split 10/10, append a mod-36 char checksum to each half → 22 chars (alphabet `abcdefghijklmnopqrstuvwxyz0123456789`); plaintext = `key_hash + "1" + "g" + hex(counter) + "y" + [custom] + "z"`; then a reversible 2-register salt cipher (2 random alphabet chars + a check char prefixed) → 30-char label, e.g. `9mwybk1vfyd9ix69ct06dlu5cwim6b`.

2. **Poll.** `GET https://polling.oastify.com/burpresults?biid=<urlencoded-base64-secret>` (legacy host `polling.burpcollaborator.net`). Response is **JSON, not Java-serialized**: `{"responses":[{"protocol","opCode","interactionString","clientPart","data":{...},"time","client"}]}`; empty poll → `{}`. `data.request`/`data.response` are base64; DNS carries `subDomain`/`type`. Interactions are DELETED on read.

3. **DNS retrieval:** none. Retrieval is HTTP-only.

4. **Q5 (biid semantics).** `biid` is NOT the subdomain; it is base64 of the 32-byte secret. The server hashes the submitted `biid` and returns interactions whose identifiers derive from it — so any self-generated secret works with no prior registration, but you can only retrieve interactions whose labels you derived from that secret. A random `abc123.oastify.com` is not retrievable.

Public vs self-hosted: the public server stores interactions encrypted in Redis (user secret + master secret); the private/self-hosted server stores them in ephemeral process memory. The `nccgroup/CollaboratorPlusPlus` AES256-CBC body is that project's own private-server auth layer, NOT the public protocol.

## Sources
- https://portswigger.net/burp/documentation/collaborator/server/security (secret→hash→identifier; server hashes submitted secret; Redis/private-memory)
- https://portswigger.net/burp/extender/api/burp/iburpcollaboratorinteraction.html (fields: interaction_id, type, client_ip, time_stamp; DNS query_type/raw_query; HTTP protocol/request/response, base64)
- https://www.richardosgood.com/posts/burp-suite-collaborator-recovery/ (live plain-GET capture + JSON response + consumed-on-read, 2022)
- https://github.com/Groppoxx/OAST-Community (working public-oastify client; full id algorithm)
- https://gist.github.com/ryarmst/8ff5838a24741fcd38481db1452c9a95 (id algorithm writeup)
- https://github.com/projectdiscovery/collaborator/blob/master/burp.go (BURP_URL constant, JSON struct)
- https://github.com/jaeles-project/jaeles core/detector.go (PollCollab: polling.burpcollaborator.net/burpresults?biid=, JSON "responses")
- https://github.com/isira-adithya/bcollabtodiscord (polling.oastify.com/burpresults vs old burpcollaborator.net)
- https://github.com/nccgroup/CollaboratorPlusPlus (private-server auth proxy)

## Open questions
- Exact encoding (base36/SHA1 + salt cipher) is community-reverse-engineered, not from PortSwigger source → medium confidence.
- Whether the server validates the check chars/version on inbound labels (rejecting malformed `abc123`) is unverified.
- Whether the public polling endpoint now also enforces rate limits / requires a valid `biid` format beyond base64-of-32-bytes: unverified.
