---
title: "rs_search mirror header relocation"
---

# rs_search mirror header relocation

# rs_search mirror header relocation

Keywords: rs_search, discovery, zg, zvec-grep, mirror, //! header, chunker, catalog metadata, needsIndex

zg's Rust chunker attaches a file-level `//!` doc header to the first symbol chunk; a non-symbol item (a `use` declaration) sitting between the header and that symbol breaks the attachment and the header text never enters the index - on either leg (fts or vector). Every rs piece has exactly that shape (`//!` header, `use pi_rust_lib`, `fn main`), so catalog metadata (name/description/keywords/args) was invisible and discovery only worked when a query term also appeared in code.

The pi-rs side fixes it at the mirror, not the source: the mirror is never compiled, so `mirroredContent()` relocates the leading `//!` run (shebang stays on top) to sit directly on the first top-level `fn`, and `mirrorCatalog()` writes that form with `writeFileSync` only when the content differs from the existing mirror file. Sources stay rust-script-shaped; the transform is idempotent, so a settled mirror rewrites to itself and every search reports `needsIndex: false` (the steady-state no-op that keeps the zg index from rebuilding on every query).

- discriminating test: a token that lives only in a header (postex_dns_poll's `消费件` appears only on its `//! description` and `//! keywords` lines) recalls its own piece at rank 1, and a suite's own CJK family words fill the top results (postex takes 6 of the top 8 for `后渗透套件 通道`)
- the principled fix belongs in the zg engine (preamble chunking before the first symbol regardless of intervening `use` items); the mirror transform is the pi-rs-side restoration
- observables live in the rust-tools debug log: `search.zg-raw` carries `needsIndex` and the top hit, `search.call` carries `matches`/`engine`/`ms`; a `needsIndex: true` call costs roughly 1.5-1.7s against ~0.6s settled

## Links

- rides: [[crates/zvec-grep]]
- grounds: [[tools/script-plane]]
- depends-on: [[crates/rust-script]]
