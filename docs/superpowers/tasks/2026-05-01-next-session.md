# Next Session Handoff

## Goal

Extend AI Tracker beyond OpenAI by implementing the next real provider connectors and improving background sync.

## Current State

- Tauri 2 + Vue 3 + TypeScript frontend is already replacing the default template.
- Tailwind CSS 4 and lucide-vue-next are configured.
- Product and architecture docs exist in `PRODUCT.md`, `DESIGN.md`, and `docs/superpowers/specs/2026-05-01-ai-tracker-design.md`.
- Secure local credential storage is implemented for OpenAI via OS keyring.
- Local SQLite persistence is implemented for:
  - OpenAI settings metadata
  - usage snapshots
  - sync events
- OpenAI connector is real:
  - validates API key via `GET /v1/models`
  - attempts usage sync via organization usage/cost endpoints
  - persists warning if the key is valid but lacks Admin Key permissions
- Dashboard UI already reads persisted backend state through Tauri commands.

## Important Constraint

OpenAI usage and costs usually require an organization Admin Key. A normal API key may validate correctly but still fail to read usage. The app already handles this as a warning and should continue doing so.

## Recommended Next Task

Implement Anthropic as the second real provider, following the same pattern as OpenAI:

1. Add secure credential storage in keyring.
2. Add provider settings metadata in SQLite.
3. Add Anthropic validation command.
4. Add Anthropic sync command and persistence.
5. Surface connection state and sync warnings in the dashboard UI.

## After Anthropic

1. Implement Gemini with the same pattern.
2. Add polling configuration and automatic refresh.
3. Improve provider-specific connection panels so the dashboard is no longer OpenAI-only.
4. Add monthly/history views and export.

## Acceptance Criteria For Next Session

- Anthropic credentials can be saved securely without exposing secrets to the frontend.
- Anthropic connection metadata is stored in SQLite.
- Anthropic usage sync either:
  - stores real usage data when the API supports it, or
  - stores a clear persisted warning if the account/credentials cannot expose usage.
- Dashboard reflects Anthropic connection state and latest sync result.
- `pnpm build` passes.
- `cargo check` passes.

## Files To Read First

- `PRODUCT.md`
- `DESIGN.md`
- `docs/superpowers/specs/2026-05-01-ai-tracker-design.md`
- `src-tauri/src/lib.rs`
- `src-tauri/src/openai.rs`
- `src-tauri/src/security.rs`
- `src-tauri/src/storage.rs`
- `src-tauri/src/providers.rs`
- `src/components/dashboard/OpenAiSetupPanel.vue`
- `src/composables/useDashboardData.ts`

## Useful Validation Commands

```bash
pnpm build
rtk cargo check
```

## Suggested Prompt For The Next Session

```text
Continua AI Tracker desde el estado actual. Ya existe Tauri + Vue + Tailwind 4, keyring local, SQLite local y un conector real para OpenAI. Implementa Anthropic con el mismo patron: credenciales seguras, metadata persistida, validacion, sync real o warning persistido si la API no expone usage, y actualiza el dashboard para reflejar ese estado. Mantén los componentes Vue por debajo de 200 líneas y verifica con pnpm build y cargo check.
```
