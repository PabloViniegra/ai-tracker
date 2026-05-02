# OpenAI Connector Simplification Design

## Problem

The OpenAI connector asks for account label, organization ID, and project ID before users can learn whether token usage and costs are available. This makes the first-run task feel heavier than necessary because only the API key is required to validate access and attempt usage sync.

## Approved Direction

Use a one-field primary setup flow. The user enters an OpenAI Admin API key, then the app validates credentials and automatically attempts the initial usage/cost sync. Optional metadata stays available, but moves behind an advanced disclosure so it does not block first-run discovery.

## UI Behavior

- Primary field: `Admin API key`.
- Primary action: `Connect and check usage`.
- Advanced disclosure contains `Account label`, `Organization ID`, and `Project ID`.
- The panel explains that regular API keys can validate but may not expose organization usage/costs.
- Connection status remains visible after save so users can see credentials, usage access, validation, sync, and warnings.
- The separate first-time `Sync OpenAI` button is removed from this panel because saving already performs an initial sync.

## Backend Behavior

No backend contract change is required. The existing `save_openai_credentials` command already validates, stores the key, persists optional metadata, and attempts initial sync. Empty advanced fields continue to be sent as `null`.

## Component Boundary

Keep the change inside `OpenAiSetupPanel.vue`. Reduce duplicated modal/non-modal markup and keep the component under 200 lines. Do not introduce new global state or Pinia stores.

## Testing

- Run `pnpm test` to verify existing provider tests.
- Run `pnpm build` to verify Vue type-checking and production build.
- Run `cargo check` in `src-tauri` if frontend changes touch Tauri contracts.
