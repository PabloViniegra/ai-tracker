# AI Tracker Design

## Architecture

AI Tracker uses Tauri 2 with Vue 3 and TypeScript on the frontend, and Rust commands for local application services.

```text
Vue UI
  -> Tauri commands
    -> Rust provider registry
    -> Credentials vault
    -> Usage normalization
    -> Local storage
    -> Scheduler and sync jobs
```

The MVP starts with mock provider data behind Rust commands so the frontend contract is real before external APIs are connected.

## Backend Boundaries

- `domain`: serializable provider, usage, sync, and capability models.
- `providers`: provider registry and mock usage snapshots.
- Future `credentials`: secure storage via Windows keyring/DPAPI.
- Future `storage`: SQLite history for snapshots, errors, and preferences.
- Future `sync`: rate limits, retries, polling, and manual refresh orchestration.

## Frontend Boundaries

- `App.vue`: app shell composition only.
- `components/dashboard`: focused presentation components under 200 lines.
- `composables/useDashboardData.ts`: loads Tauri data and exposes readonly dashboard state.
- `types/usage.ts`: frontend contracts matching Rust command payloads.

## Usage Normalization

Each snapshot exposes a common shape:

- provider id and account id
- daily and weekly token totals
- input, output, and cached tokens when available
- request count
- cost in USD when available
- quota used and limit when available
- source: `official_api`, `local_estimate`, or `manual`
- confidence: `high`, `medium`, or `low`

## Provider Strategy

- OpenAI and Anthropic are the only active providers in the current product scope.
- Additional providers can return later, but should not appear in the dashboard or shared contracts until their connector path is intentionally restored.

## Visual System

Intent: a private technical ledger for developers and power users who need to know whether AI subscriptions are under control.

Palette: cold blue data surfaces for trust and telemetry, amber highlights for budget pressure, slate neutrals for a desktop analytics feel.

Depth: borders-first with subtle surface shifts. This keeps the app precise and avoids glossy generic SaaS cards.

Typography: Fira Sans for interface copy and Fira Code for token numbers, provider identifiers, and telemetry labels.

Spacing: 4px base rhythm, with compact 8/12/16/24px groupings for a data-dense dashboard.

Signature: token ledger strips. Provider cards include a small segmented token rail that makes usage feel metered instead of decorative.

## Implementation Notes

- Do not store credentials in frontend state beyond transient form input.
- Avoid `localStorage` for secrets.
- Keep provider capabilities explicit so the UI can show unsupported metrics gracefully.
- Prefer typed discriminated unions for source and confidence values.
- Keep Vue components focused and below 200 lines.
