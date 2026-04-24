# Omitl

> *Del náhuatl: hueso / estructura.*

CLI tool written in Rust that generates corporate API contract documentation in **PDF** and **DOCX** from a simple JSON file. Designed to be the official skeleton of communication between developers and clients.

```bash
omitl generate --input api.json --brand brand.json --format pdf
```

---

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) 1.85+
- [Typst](https://typst.app) — for PDF generation (`sudo pacman -S typst` on Arch)

---

## Installation

**From source:**
```bash
git clone https://github.com/yourusername/omitl
cd omitl
./omitl build
```

The binary will be at `target/release/omitl`.

**Arch Linux (AUR) — coming soon:**
```bash
yay -S omitl
```

---

## Usage

```bash
# Generate PDF from a contract file
omitl generate --input api.json

# With corporate branding
omitl generate --input api.json --brand brand.json

# Generate DOCX instead
omitl generate --input api.json --brand brand.json --format docx

# Import from an OpenAPI / Swagger spec
omitl generate --input swagger.json --openapi

# Validate a contract without generating output
omitl validate --input api.json
```

See [`examples/`](./examples) for sample `api_contract.json` and `brand.json` files.

---

## Development

### Linux / macOS

Use the `./omitl` script included in the repo — no extra tools required.

```bash
./omitl run              # compile and run
./omitl run generate --input examples/api_contract.json
./omitl build            # release binary → target/release/omitl
./omitl check            # type-check only (fast)
./omitl test             # run test suite
./omitl fmt              # format code
./omitl lint             # clippy linter
./omitl example          # generate sample PDF from examples/
./omitl help             # show all commands
```

### Windows

Use `omitl.cmd` from the project root in CMD or PowerShell:

```bat
omitl run
omitl run generate --input examples/api_contract.json
omitl build
omitl check
omitl test
omitl fmt
omitl lint
omitl example
omitl help
```

### Alternative: Justfile

If you have [`just`](https://just.systems) installed, every command is also available as `just <recipe>` with two extras:

```bash
just ci            # fmt + lint + test in one shot
just clean         # remove target/ artifacts
```

---

## Contract format

A contract is a JSON (or YAML) file describing your API:

```json
{
  "title": "Payments API",
  "version": "1.0.0",
  "base_url": "https://api.example.com/v1",
  "endpoints": [
    {
      "method": "post",
      "path": "/payments",
      "title": "Create Payment",
      "parameters": [
        { "name": "amount", "location": "body", "type": "number", "required": true }
      ],
      "responses": [
        { "status": 201, "description": "Payment created." }
      ]
    }
  ]
}
```

Endpoints with no parameters render a **Ninguno** row in the table — the document structure is always preserved.

OpenAPI 3.x specs are also accepted via the `--openapi` flag.

---

## Brand config

Visual identity lives in a separate JSON so content and design never mix:

```json
{
  "company_name": "Acme Corp",
  "primary_color": "#1A3C5E",
  "secondary_color": "#F5F5F5",
  "accent_color":   "#E8631A",
  "font_family":    "Liberation Sans",
  "footer_text":    "Acme Corp — Confidential",
  "logo": {
    "data":     "<base64-encoded image>",
    "mime":     "image/png",
    "position": "left"
  }
}
```

Logos are embedded as Base64 so brand files are fully self-contained and portable.

---

## License

MIT
