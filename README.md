# AI Tracker

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-24C8DB?logo=tauri&logoColor=white&style=for-the-badge" alt="Tauri" />
  <img src="https://img.shields.io/badge/Vue.js-4FC08D?logo=vuedotjs&logoColor=white&style=for-the-badge" alt="Vue.js" />
  <img src="https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white&style=for-the-badge" alt="TypeScript" />
  <img src="https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white&style=for-the-badge" alt="Rust" />
  <img src="https://img.shields.io/badge/Tailwind_CSS-06B6D4?logo=tailwindcss&logoColor=white&style=for-the-badge" alt="Tailwind CSS" />
  <img src="https://img.shields.io/badge/Vite-646CFF?logo=vite&logoColor=white&style=for-the-badge" alt="Vite" />
  <img src="https://img.shields.io/badge/Vitest-729B1B?logo=vitest&logoColor=white&style=for-the-badge" alt="Vitest" />
</p>

A privacy-first desktop application for tracking AI subscription token usage locally. AI Tracker currently aggregates daily and weekly usage data from OpenAI and Anthropic, with no cloud backend.

## Table of Contents

- [Features](#features)
- [Supported Providers](#supported-providers)
- [Tech Stack](#tech-stack)
- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
  - [Installation](#installation)
  - [Development](#development)
  - [Build](#build)
- [Project Structure](#project-structure)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Local-first**: All data stays on your machine. No cloud backend in the MVP.
- **Focused provider tracking**: Monitor token usage from OpenAI and Anthropic from a single dashboard.
- **Transparency**: Every usage value includes a source label and confidence level (official API, local estimate, or manual).
- **Secure credentials**: Secrets stored via OS keyring (Windows DPAPI), never in frontend state.
- **Historical trends**: Daily and weekly usage snapshots retained locally for trend analysis.
- **Modular connectors**: Provider plugins can differ in capabilities without breaking the dashboard.

## Supported Providers

| Provider | Status |
|----------|--------|
| OpenAI | Active |
| Anthropic | Active |

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop framework | Tauri 2 |
| Frontend | Vue 3 (Composition API, `<script setup>`) |
| Language | TypeScript 5.6 |
| Styling | Tailwind CSS 4 |
| Build tool | Vite 6 |
| Testing | Vitest |
| Backend | Rust (edition 2021) |
| Database | SQLite (rusqlite) |
| HTTP client | reqwest |
| Credential storage | keyring |

## Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) >= 8
- [Rust](https://www.rust-lang.org/tools/install) >= 1.70
- Platform-specific Tauri dependencies (see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))

## Getting Started

### Installation

```bash
# Clone the repository
git clone https://github.com/your-username/ai-tracker.git
cd ai-tracker

# Install frontend dependencies
pnpm install

# Install Rust dependencies (automatic via cargo)
```

### Development

```bash
# Start the development server with hot reload
pnpm tauri dev
```

### Build

```bash
# Type-check and build for production
pnpm build

# Build the native desktop application
pnpm tauri build
```

The compiled binaries will be placed in `src-tauri/target/release/bundle/`.

## Project Structure

```
ai-tracker/
├── src/                    # Vue frontend
│   ├── components/         # UI components (< 200 lines each)
│   │   └── dashboard/      # Dashboard-specific components
│   ├── composables/        # Vue composables (data loading, state)
│   └── types/              # TypeScript type definitions
├── src-tauri/              # Rust backend
│   ├── src/                # Rust source code
│   │   ├── domain/         # Serializable models
│   │   └── providers/      # Provider registry and snapshots
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── public/                 # Static assets
├── index.html              # Entry HTML
├── vite.config.ts          # Vite configuration
├── tsconfig.json           # TypeScript configuration
└── package.json            # Node dependencies and scripts
```

## Architecture

```
Vue UI
  -> Tauri commands
    -> Rust provider registry
    -> Credentials vault (keyring)
    -> Usage normalization
    -> Local storage (SQLite)
    -> Scheduler and sync jobs
```

Each usage snapshot follows a normalized shape:

- Provider ID and account ID
- Daily and weekly token totals
- Input, output, and cached token breakdown
- Request count and cost (USD)
- Source: `official_api`, `local_estimate`, or `manual`
- Confidence: `high`, `medium`, or `low`

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to submit changes via fork.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
