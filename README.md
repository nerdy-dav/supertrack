# SuperTrack AU — Payday Super Compliance Manager

A self-hosted web application for Australian small businesses to track
Payday Super obligations (effective 1 July 2026).

[Download the latest release](https://github.com/your-org/supertrack/releases/latest)
— compiled binaries for Linux, macOS (Apple Silicon), and Windows. No Rust or
build tools needed.

## Features

- **7-business-day deadline engine** — accounts for national + state public holidays
- **Compliance status tracking** — Compliant / At Risk / Overdue / Paid Late
- **Cash flow impact calculator** — models the working capital shift from quarterly to per-payday super
- **SBSCH migration checklist** — step-by-step guide for businesses leaving the ATO clearing house
- **CSV export** — ATO audit-ready compliance ledger
- **Zero dependencies** — single binary, SQLite database, no internet required

## Quick Start

1. Download the binary for your OS from the [releases page](https://github.com/your-org/supertrack/releases/latest)
2. Run it from a terminal:
   ```bash
   ./supertrack
   ```
3. Open http://localhost:3000 in your browser
4. Go to **Setup** and enter your business details first

## First Run Checklist

1. Go to **Setup** — enter business name, ABN, state, pay frequency, clearing house
2. Go to **Employees** — add each employee with their super fund details
3. After each pay run, go to **Pay Runs → Log Pay Run**
4. Once you've paid super, go to **Pay Runs → Record Payment**
5. Check **Dashboard** for live compliance status
6. Use **Cash Flow Calculator** to plan for the working capital shift
7. Export reports from **Reports** for your accountant

## Regulatory Notes

- Payday Super starts **1 July 2026**
- Super must reach the clearing house within **7 business days** of pay date
- The ATO's SBSCH closes **30 June 2026** — you must switch providers
- SGC penalties are NOT tax-deductible (from 1 July 2025)
- Always verify obligations with your registered tax agent

## Data & Privacy

All data is stored locally in `supertrack.db` (SQLite) in the same directory
as the binary. No data is sent anywhere. Back this file up regularly.

## Disclaimer

This tool assists with record-keeping only. It is not financial or legal advice.
Always verify compliance obligations with a registered tax agent or BAS agent.

---

## Development

### Requirements

- Rust 1.70+ (`rustup` from rustup.rs)
- Linux / macOS / Windows

### Build & Run

```bash
git clone <repo-url>
cd supertrack
cargo build --release
./target/release/supertrack
```

Or for development with hot-reload (using `cargo-watch`):
```bash
cargo watch -x run
```

### Run Tests

```bash
cargo test
```

### Project Layout

```
src/
  main.rs              — Entry point, router setup
  db/mod.rs            — SQLite database layer
  domain/
    deadline.rs        — Business day & public holiday calculations
    compliance.rs      — Compliance status determination
    cashflow.rs        — Cash flow impact analysis
  routes/              — Axum route handlers
templates/             — MiniJinja HTML templates
static/                — CSS assets
```

### CI / Release

Push and PR events run `cargo test` via GitHub Actions. Creating a release
triggers a matrix build across Linux, macOS (ARM), and Windows — each
binary is zipped and attached to the release automatically.
