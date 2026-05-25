# SuperTrack AU — Payday Super Compliance Manager

A self-hosted Rust application for Australian small businesses to track 
Payday Super obligations (effective 1 July 2026).

## Features

- **7-business-day deadline engine** — accounts for national + state public holidays
- **Compliance status tracking** — Compliant / At Risk / Overdue / Paid Late
- **Cash flow impact calculator** — models the working capital shift from quarterly to per-payday super
- **SBSCH migration checklist** — step-by-step guide for businesses leaving the ATO clearing house
- **CSV export** — ATO audit-ready compliance ledger
- **Zero dependencies** — single binary, SQLite database, no internet required

## Requirements

- Rust 1.70+ (`rustup` from rustup.rs)
- Linux / macOS / Windows

## Quick Start

```bash
# 1. Clone or extract this project
cd supertrack

# 2. Build the release binary
cargo build --release

# 3. Run
./target/release/supertrack

# 4. Open http://localhost:3000 in your browser
# 5. Go to Setup and enter your business details first
```

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

## Self-Hosting (Production)

```bash
# Run as systemd service (Linux)
cargo build --release
sudo cp target/release/supertrack /usr/local/bin/
# See systemd/supertrack.service for unit file example
```

## Disclaimer

This tool assists with record-keeping only. It is not financial or legal advice.
Always verify compliance obligations with a registered tax agent or BAS agent.
