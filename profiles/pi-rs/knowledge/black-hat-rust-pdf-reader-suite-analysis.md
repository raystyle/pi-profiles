---
title: "Black Hat Rust PDF reader-suite analysis"
---

# Black Hat Rust PDF reader-suite analysis

# Black Hat Rust PDF — reader-suite analysis

## Question
深度研究 `/mnt/d/Black Hat Rust.pdf` with the reader suite (doc_read → doc_outline → doc_stats); report title, chapter count, top-3 terms, reading minutes; report format limits honestly if the text layer is unreadable.

## Answer (receipt)
- **Title**: *Black Hat Rust — Applied offensive security with the Rust programming language* — Sylvain Kerkour, `v2022.56` (PDF `/Title` = "Black Hat Rust"; producer `itext-paulo-155` / creator `pdftk-java 3.0.9`).
- **Pages**: 358 (`pdfinfo`). Note: `file(1)` misreports "12 page(s)" — trust `pdfinfo`.
- **Chapters**: 14 top-level chapters (1 Introduction … 14 Conclusion) + 4 front-matter items (Copyright, Your early access bonuses, Contact, Preface). Outline totals **217 headings** = 1 title + 14 chapters + 202 sections/` x.y.z` subsections.
- **Top-3 terms** (doc_stats, doc_read output): `rust` 375, `error` 251, `string` 239.
- **Reading minutes**: **168** (`doc_stats` on doc_read markdown, 36,909 counted words @220 wpm). 169 on the raw pdftotext dump.

## Format / tool limitations (verified)
- **The PDF text layer IS readable** — not a scanned image: 445 `/Font` objects; `pdftotext -layout` yields 16,365 lines / 72,964 tokens / 520,975 chars; producer `itext` (born-digital).
- **Bundled `doc_read` v1.0.0 does NOT compile here**: calls `mdka::from_html(&cleaned)`, but installed `mdka 3.2.0` exposes `mdka::html_to_markdown` (verified in `~/.cargo/registry/.../mdka-3.2.0/src/lib.rs:79`). First symptom was a misleading `rustc ... could not locate working directory`.
- **Bundled `doc_outline` v1.0.0 does NOT compile**: references `serde_json::Value` / `serde_json::json!` with no `serde_json` dep (only `pi_rust_lib::serde_json` imported).
- **`doc_read` is inherently PDF-blind**: it reads via `fs::read_to_string` and only converts HTML (`mdka`), so a binary PDF would fail UTF-8 read even after the mdka fix.
- **`doc_stats` v1.0.0 works** as-is (pure text).
- **Workaround applied**: project-tier overrides at `.pi-rs/rust-scripts/doc_read.rs` (mdka fix + `pdftotext` PDF branch, `kind:"pdf"`) and `.pi-rs/rust-scripts/doc_outline.rs` (serde_json fix). Both now run; doc_read output `/tmp/bhr.md`, outline source `/tmp/bhr_toc.md` (built from the PDF Contents).

## Sources
- `/mnt/d/Black Hat Rust.pdf` (4,092,455 bytes, PDF 1.7, 358 pp)
- `pdfinfo`/`pdftotext` (poppler) output; extracted text `/tmp/bhr.txt`
- `~/.cargo/registry/src/rsproxy.cn-*/mdka-3.2.0/src/lib.rs` (API proof)
- `.pi-rs/rust-scripts/doc_read.rs`, `.pi-rs/rust-scripts/doc_outline.rs` (repairs)

## Open questions
- Whether the bundled mdka breakage is a version-pin regression (script pins `mdka = "3.2"` but calls a 2.x-era API).
- `doc_find` was not exercised (same reader family; likely fine, text-only).
