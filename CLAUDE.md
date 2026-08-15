# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Omitl is a Rust CLI that generates corporate API contract documentation (PDF/DOCX) from a JSON/YAML contract file, or auto-generates that contract by scanning a project's source code. Contract content and visual branding are deliberately kept in separate files (`contracts/*.json` vs brand configs) so they never mix.

## Commands

Use the `./omitl` dev script (or `just <recipe>`, or `omitl.cmd` on Windows) — not raw `cargo` — since it wraps the same recipes:

```bash
./omitl run              # cargo run --
./omitl build             # release binary → target/release/omitl
./omitl check             # cargo check (fast type-check)
./omitl test              # cargo test
./omitl fmt               # cargo fmt
./omitl lint              # cargo clippy -- -D warnings
./omitl example           # generate sample PDF from contracts/payments-api.json
./omitl batch             # generate PDF for every contract in contracts/
```

Full pre-commit pipeline: `just ci` (fmt + lint + test).

Run a single test: `cargo test <test_name>`.

**External dependency:** the `typst` CLI must be installed and on `PATH` — PDF rendering shells out to `typst compile` (see `src/render/pdf.rs`). This is not vendored; without it, PDF generation fails with a clear error but DOCX generation and `validate`/`scan` still work.

## Architecture

Pipeline for `omitl generate`: **load contract → validate → load brand → render**.

- `src/schema/contract.rs` — `ApiContract`/`Endpoint`/`Parameter` are the canonical data model (serde-derived, JSON/YAML interchangeable). Everything downstream (rendering, scanning) produces or consumes this struct.
- `src/schema/openapi.rs` — converts an OpenAPI 3.x spec into an `ApiContract` (`--openapi` flag).
- `src/schema/validate.rs` — structural validation before rendering.
- `src/config/brand.rs` — `BrandConfig` (colors, font, logo, footer) is a fully separate struct from `ApiContract`; logos are embedded as Base64 so brand files are self-contained/portable.
- `src/render/pdf.rs` — renders `templates/contract.typ.tera` (a Tera template producing Typst markup) with a context built in `src/render/context.rs`, then shells out to the external `typst` binary to compile the `.typ` source to PDF. A custom `typst_json` Tera filter escapes JSON values for safe embedding in Typst source. Logo/watermark images are base64-decoded to temp files so Typst can reference them by path.
- `src/render/docx.rs` — separate, independent DOCX renderer (via `docx-rs`), does not go through Typst.
- `src/scanner/` — the `scan` command auto-detects a web framework from project files (`go.mod`, `package.json`, `requirements.txt`/`pyproject.toml`) via `detect.rs`, then produces an `ApiContract` without touching the scanned project's source. Currently only Go frameworks (Fiber/Gin/Echo) have an implemented scanner (`go.rs`); Node/Python detection exists but scanning for them is not yet implemented — it errors out and suggests exporting an OpenAPI spec instead.

## Conventions

- Endpoints with no parameters must still render a "Ninguno" row in the output table — this is intentional, not a bug; the document structure is always preserved (see `Endpoint::has_parameters`).
- Contract JSON files live in `contracts/` (one file per API); generated output goes to `output/<api-name>/` and is gitignored. This supports the multi-API batch workflow (`./omitl batch`).
- Brand/content separation is a hard rule: never fold visual identity fields into `ApiContract`, or content fields into `BrandConfig`.
