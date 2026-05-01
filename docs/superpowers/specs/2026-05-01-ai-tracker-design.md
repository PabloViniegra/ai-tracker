# AI Tracker MVP Design Specification

## Approved Direction

Build a Windows desktop app with Tauri 2, Vue 3, TypeScript, and Rust. The app is local-first and private: no backend cloud is included in the MVP.

## Scope

The MVP tracks AI subscription token usage across providers through a normalized provider connector layer. Official APIs are preferred for OpenAI, Anthropic, and Gemini. Providers without reliable usage APIs start as unsupported or experimental until a trustworthy connector is confirmed.

## Architecture

```text
Vue dashboard
  -> Tauri commands
    -> Rust provider registry
    -> Future credential vault
    -> Future SQLite storage
    -> Future sync scheduler
```

The first implementation uses mock provider data from Rust commands so the UI and backend contract are real. Future provider implementations can replace mock snapshots without changing component contracts.

## Provider Model

Providers declare capabilities for tokens, cost, quota, realtime, and historical data. Usage values include a source and confidence level so the UI does not imply exactness for estimated integrations.

Supported source values:

- `official_api`
- `local_estimate`
- `manual`

Supported confidence values:

- `high`
- `medium`
- `low`

## UI Design System

The interface is a technical token ledger for developers and power users. It uses data-dense dashboard patterns, slate/blue surfaces, amber budget highlights, Fira Sans for interface copy, and Fira Code for numerical telemetry.

The signature element is a metered token rail on provider cards, indicating usage pressure without pretending every provider has the same quota semantics.

## Component Boundaries

- `App.vue`: shell composition and data loading only.
- `AppSidebar.vue`: navigation and privacy status.
- `HeroMetrics.vue`: aggregate KPIs and manual sync action.
- `ProviderGrid.vue`: provider status, capability badges, and token rails.
- `UsagePanel.vue`: weekly usage visualization.
- `SyncTimeline.vue`: connector events and warnings.
- `useDashboardData.ts`: Tauri command integration and derived totals.

## Next Implementation Steps

- Add secure credential storage through Windows keyring/DPAPI.
- Add SQLite for local historical snapshots and preferences.
- Implement the first real provider connector, starting with OpenAI.
- Add credential setup forms and validation commands.
- Add polling configuration and persisted sync events.
