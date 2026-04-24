# Omitl

> *Del náhuatl: hueso / estructura.*

CLI tool written in Rust that generates corporate API contract documentation in **PDF** and **DOCX** from a simple JSON file. Designed to be the official skeleton of communication between developers and clients.

```bash
omitl generate --input api.json --brand brand.json --format pdf
```

---

## Installation

### Linux / macOS — one-liner

```bash
curl -fsSL https://raw.githubusercontent.com/yourusername/omitl/main/install.sh | sh
```

### Windows — PowerShell

```powershell
irm https://raw.githubusercontent.com/yourusername/omitl/main/install.ps1 | iex
```

### Package managers

| Platform | Command |
|---|---|
| Arch Linux | `yay -S omitl` *(AUR — coming soon)* |
| macOS / Linux | `brew install yourusername/tap/omitl` *(Homebrew — coming soon)* |
| Windows | `scoop install omitl` *(Scoop — coming soon)* |
| Any (Rust) | `cargo install omitl` |

### Manual — download binary

Go to [Releases](https://github.com/yourusername/omitl/releases) and download the binary for your platform:

| File | Platform |
|---|---|
| `omitl-linux-x86_64.tar.gz` | Linux 64-bit |
| `omitl-linux-arm64.tar.gz` | Linux ARM (Raspberry Pi, etc.) |
| `omitl-macos-x86_64.tar.gz` | macOS Intel |
| `omitl-macos-arm64.tar.gz` | macOS Apple Silicon (M1/M2/M3) |
| `omitl-windows-x86_64.zip` | Windows 64-bit |

Extract and place the binary somewhere in your `PATH`.

### Build from source

Requires [Rust 1.85+](https://rustup.rs) and [Typst](https://typst.app).

```bash
git clone https://github.com/yourusername/omitl
cd omitl
cargo build --release
# binary → target/release/omitl  (target\release\omitl.exe on Windows)
```

> **Note:** Typst is currently required as an external dependency for PDF generation.
> This will be removed in v0.2 — the binary will be fully self-contained.

---

## Requirements

Omitl has **one external dependency** today: [Typst](https://typst.app), used for PDF rendering.
It will be eliminated in v0.2 (compiled into the binary).

| Platform | Install Typst |
|---|---|
| Arch Linux | `sudo pacman -S typst` |
| Ubuntu / Debian | `sudo snap install typst` |
| macOS | `brew install typst` |
| Windows | `winget install Typst.Typst` |
| Any | `cargo install typst-cli` |

**Contributing only** — [`just`](https://just.systems) task runner:

| Platform | Install just |
|---|---|
| Arch Linux | `sudo pacman -S just` |
| Ubuntu / Debian | `sudo apt install just` |
| macOS | `brew install just` |
| Windows | `winget install Casey.Just` |
| Any | `cargo install just` |

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

See [`contracts/`](./contracts) for sample contract files and [`examples/`](./examples) for a sample brand config.

---

## Multi-API workflow

Place one contract file per API inside `contracts/`, then generate all at once.
Output goes to `output/<api-name>/contract.pdf` — this directory is gitignored.

```bash
contracts/
  payments-api.json
  users-api.json
  inventory-api.json
```

**Generate all:**

```bash
# Linux / macOS
./omitl batch

# Windows
omitl.cmd batch

# With just
just batch
```

**With OpenAPI specs** (FastAPI, Express, Gin, etc.):

```bash
# Export from your running API
curl http://localhost:8000/openapi.json > contracts/my-api.json

# Generate with --openapi flag
omitl generate --input contracts/my-api.json --openapi --brand examples/brand.json
```

---

## Development

### Linux / macOS

Use the `./omitl` script included in the repo:

```bash
./omitl run              # compile and run
./omitl build            # release binary → target/release/omitl
./omitl check            # type-check only (fast)
./omitl test             # run test suite
./omitl fmt              # format code
./omitl lint             # clippy linter
./omitl example          # generate sample PDF from contracts/payments-api.json
./omitl batch            # generate PDF for every contract in contracts/
./omitl help             # show all commands
```

### Windows

Use `omitl.cmd` from the project root in CMD or PowerShell:

```bat
omitl run
omitl build
omitl check
omitl test
omitl example
omitl batch
omitl help
```

### Alternative: just

If `just` is installed, every command is also available as `just <recipe>`:

```bash
just run
just build
just batch           # generate all contracts
just ci              # fmt + lint + test in one shot
just clean           # remove target/ artifacts
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
